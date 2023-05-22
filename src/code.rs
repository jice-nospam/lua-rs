//! Code generator for Lua

use crate::{
    api::LuaError,
    lex::LexState,
    opcodes::{
        create_abc, create_abx, get_arg_b, get_arg_c, get_arg_sbx, get_opcode, is_reg_constant,
        set_arg_a, OpCode, NO_JUMP, NO_REG, set_arg_c, set_arg_b,
    },
    parser::{ExpressionDesc, ExpressionKind, FuncState, UnaryOp, BinaryOp},
    LuaNumber, LUA_MULTRET, limits::MAX_LUA_STACK,
};

pub(crate) fn discharge_vars<T>(
    lex: &mut LexState<T>,
    fs: &mut FuncState<T>,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    match exp.k {
        ExpressionKind::VLOCAL => exp.k = ExpressionKind::VNONRELOC,
        ExpressionKind::VUPVAL => {
            exp.info = code_abc(lex, fs, OpCode::GetUpVal as u32, 0, exp.info, 0);
            exp.k = ExpressionKind::VRELOCABLE;
        }
        ExpressionKind::VGLOBAL => {
            exp.info = code_abx(lex, fs, OpCode::GetGlobal as u32, 0, exp.info);
            exp.k = ExpressionKind::VRELOCABLE;
        }
        ExpressionKind::VINDEXED => todo!(),
        ExpressionKind::VCALL | ExpressionKind::VVARARG => todo!(),
        _ => (), // there is one value available (somewhere)
    }
    Ok(())
}

pub(crate) fn code_abc<T>(
    lex: &mut LexState<T>,
    fs: &mut FuncState<T>,
    op: u32,
    a: u32,
    b: u32,
    c: u32,
) -> u32 {
    let o = create_abc(op, a, b, c);
    code(fs, o, lex.lastline)
}

pub(crate) fn code_abx<T>(lex: &mut LexState<T>, fs: &mut FuncState<T>, op: u32, a: u32, bx: u32) -> u32 {
    let o = create_abx(op, a, bx);
    code(fs, o, lex.lastline)
}

fn code<T>(fs: &mut FuncState<T>, o: u32, line: usize) -> u32 {
    discharge_jpc(fs); // pc' will change
    let f = &mut fs.f;
    let pc = f.code.len() as u32;
    f.code.push(o);
    f.lineinfo.push(line);
    pc
}

fn discharge_jpc<T>(fs: &mut FuncState<T>) {
    let pc = fs.f.code.len() as i32;
    patch_list_aux(fs, fs.jpc, pc, NO_REG, pc);
    fs.jpc = NO_JUMP;
}

fn patch_list_aux<T>(fs: &mut FuncState<T>, jpc: i32, vtarget: i32, reg: u32, dtarget: i32) {
    let mut jpc = jpc;
    while jpc != NO_JUMP {
        let next = get_jump(fs, jpc);
        if patch_test_reg(fs, jpc, reg) {
            fix_jump(fs, jpc, vtarget);
        } else {
            fix_jump(fs, jpc, dtarget);
        }
        jpc = next;
    }
}

fn fix_jump<T>(_fs: &mut FuncState<T>, _jpc: i32, _vtarget: i32) {
    todo!()
}

fn patch_test_reg<T>(fs: &mut FuncState<T>, node: i32, reg: u32) -> bool {
    let i = get_jump_control(fs, node);
    if get_opcode(*i) != OpCode::TestSet {
        return false; // cannot patch other instructions
    } else if reg != NO_REG && reg != get_arg_b(*i) {
        set_arg_a(i, reg as u32);
    } else {
        // no register to put value or register already has the value
        *i = create_abc(OpCode::Test as u32, get_arg_b(*i), 0, get_arg_c(*i));
    }
    true
}

fn get_jump_control<T>(_fs: &mut FuncState<T>, _node: i32) -> &mut u32 {
    todo!()
}

/// get the new value of pc for this jump instruction
fn get_jump<T>(fs: &mut FuncState<T>, jpc: i32) -> i32 {
    let offset = get_arg_sbx(fs.f.code[jpc as usize]);
    return if offset == NO_JUMP {
        NO_JUMP
    } else {
        jpc + 1 + offset
    };
}

pub(crate) fn exp2nextreg<T>(
    lex: &mut LexState<T>,
    fs: &mut FuncState<T>,
    exp: &mut ExpressionDesc,
) -> Result<(), LuaError> {
    discharge_vars(lex, fs, exp)?;
    free_exp(fs, exp);
    reserve_regs(lex,fs, 1)?;
    exp2reg(lex, fs, exp, fs.freereg as u32 - 1)
}

fn exp2reg<T>(
    lex: &mut LexState<T>,
    fs: &mut FuncState<T>,
    exp: &mut ExpressionDesc,
    reg: u32,
) -> Result<(), LuaError> {
    discharge2reg(lex, fs, exp, reg)?;
    if let ExpressionKind::VJMP = exp.k {
        concat_jump(fs, &mut exp.t, exp.info); // put this jump in `t' list
    }
    if has_jumps(exp) {
        let final_pc; // position after whole expression
        let mut p_f = NO_JUMP; // position of an eventual LOAD false
        let mut p_t = NO_JUMP; // position of an eventual LOAD true
        if need_value(fs, &mut exp.t) || need_value(fs, &mut exp.f) {
            let fj = if let ExpressionKind::VJMP = exp.k {
                NO_JUMP
            } else {
                jump(fs)
            };
            p_f = code_label(fs, reg, 0, 1);
            p_t = code_label(fs, reg, 1, 0);
            patch_to_here(fs, fj);
        }
        final_pc = get_label(fs);
        patch_list_aux(fs, exp.f, final_pc, reg, p_f);
        patch_list_aux(fs, exp.t, final_pc, reg, p_t);
    }
    exp.f = NO_JUMP;
    exp.t = NO_JUMP;
    exp.info = reg;
    exp.k = ExpressionKind::VNONRELOC;
    Ok(())
}

fn jump<T>(_fs: &mut FuncState<T>) -> i32 {
    todo!()
}

pub(crate) fn set_mult_ret<T>(lex: &mut LexState<T>,fs: &mut FuncState<T>, exp: &mut ExpressionDesc) -> Result<(), LuaError> {
    set_returns(lex, fs, exp, LUA_MULTRET)
}

fn set_returns<T>(lex: &mut LexState<T>,fs: &mut FuncState<T>, exp: &mut ExpressionDesc, nresults: i32) -> Result<(), LuaError> {
    if exp.k == ExpressionKind::VCALL {
        // expression is an open function call?
        let pc = exp.info as usize;
        set_arg_c(&mut fs.f.code[pc], nresults as u32+1);
    } else if exp.k == ExpressionKind::VVARARG {
        let pc = exp.info as usize;
        set_arg_b(&mut fs.f.code[pc], nresults as u32+1);
        set_arg_a(&mut fs.f.code[pc], fs.freereg as u32);
        reserve_regs(lex, fs, 1)?;
    }
    Ok(())
}

fn get_label<T>(_fs: &mut FuncState<T>) -> i32 {
    todo!()
}

fn patch_to_here<T>(_fs: &mut FuncState<T>, _fj: i32) {
    todo!()
}

fn code_label<T>(_fs: &mut FuncState<T>, _reg: u32, _arg_1: i32, _arg_2: i32) -> i32 {
    todo!()
}

fn need_value<T>(_fs: &mut FuncState<T>, _t: &mut i32) -> bool {
    todo!()
}

#[inline]
fn has_jumps(exp: &mut ExpressionDesc) -> bool {
    exp.t != exp.f
}

fn concat_jump<T>(_fs: &mut FuncState<T>, _t: &mut i32, _info: u32) {
    todo!()
}

fn discharge2reg<T>(
    lex: &mut LexState<T>,
    fs: &mut FuncState<T>,
    exp: &mut ExpressionDesc,
    reg: u32,
) -> Result<(), LuaError> {
    discharge_vars(lex, fs, exp)?;
    match exp.k {
        ExpressionKind::VNIL => nil(fs, reg as u32, 1),
        ExpressionKind::VTRUE | ExpressionKind::VFALSE => {
            code_abc(
                lex,
                fs,
                OpCode::LoadBool as u32,
                reg as u32,
                if exp.k == ExpressionKind::VTRUE { 1 } else { 0 },
                0,
            );
        }
        ExpressionKind::VK => {
            code_abx(lex, fs, OpCode::LoadK as u32, reg as u32, exp.info as u32);
        }
        ExpressionKind::VKNUM => {
            let kid = fs.number_constant(exp.nval as LuaNumber) as u32;
            code_abx(lex, fs, OpCode::LoadK as u32, reg as u32, kid);
        }
        ExpressionKind::VRELOCABLE => {
            let pc = exp.info as usize;
            set_arg_a(&mut fs.f.code[pc], reg);
        }
        ExpressionKind::VNONRELOC => {
            if reg != exp.info {
                code_abc(lex, fs, OpCode::Move as u32, reg as u32, exp.info, 0);
            }
        }
        _ => {
            debug_assert!(exp.k == ExpressionKind::VVOID || exp.k == ExpressionKind::VJMP);
            return Ok(()); //nothing to do...
        }
    }
    exp.info = reg;
    exp.k = ExpressionKind::VNONRELOC;
    Ok(())
}

fn nil<T>(_fs: &mut FuncState<T>, _reg: u32, _arg: i32) {
    todo!()
}

pub(crate) fn ret<T>(lex: &mut LexState<T>, fs: &mut FuncState<T>,first: u32, nret: u32) {
    code_abc(lex, fs, OpCode::Return as u32, first, nret+1, 0);
}

fn reserve_regs<T>(lex: &mut LexState<T>, fs: &mut FuncState<T>, count: usize) -> Result<(),LuaError> {
    check_stack(lex,fs,count)?;
    fs.freereg += count;
    Ok(())
}

pub(crate) fn check_stack<T>(lex: &mut LexState<T>, fs: &mut FuncState<T>, count: usize) -> Result<(),LuaError> {
    let new_stack = fs.freereg + count;
    if new_stack > fs.f.maxstacksize {
        if new_stack > MAX_LUA_STACK {
            return lex.syntax_error("function or expression too complex");
        }
        fs.f.maxstacksize = new_stack;
    }
    Ok(())
}

fn free_exp<T>(fs: &mut FuncState<T>, exp: &mut ExpressionDesc) {
    if let ExpressionKind::VNONRELOC = exp.k {
        free_reg(fs, exp.info);
    }
}

fn free_reg<T>(fs: &mut FuncState<T>, reg: u32) {
    if !is_reg_constant(reg as u32) && reg >= fs.nactvar as u32 {
        fs.freereg -= 1;
        debug_assert!(reg == fs.freereg as u32);
    }
}

pub(crate) fn fix_line<T>(fs: &mut FuncState<T>, line: usize) {
    let pc = fs.f.code.len();
    fs.f.lineinfo[pc-1] = line;
}

pub(crate) fn prefix<T>(_fs:  &mut FuncState<T>, _uop: UnaryOp, _exp: &mut ExpressionDesc) {
    todo!()
}

pub(crate) fn infix<T>(_lex: &mut LexState<T>,
    _fs: &mut FuncState<T>,
    _op: BinaryOp,
    _exp: &mut ExpressionDesc) {
    todo!()
}

pub(crate) fn postfix<T>(_lex: &mut LexState<T>,
    _fs: &mut FuncState<T>, _exp: &mut ExpressionDesc, _exp2: &mut ExpressionDesc) {
    todo!()
}