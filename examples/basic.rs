use lualib as lua;
use lualib::luaL;

pub fn main() {
    let mut state = luaL::newstate();
    luaL::open_libs(&mut state).unwrap();
    if let Err(_) = luaL::dostring(
        &mut state,
        "
        local sqrt = math.sqrt

        local PI = 3.141592653589793
        local DAYS_PER_YEAR = 365.24
        local bodies = {

          { -- Jupiter
            vx = 1.66007664274403694e-03 * DAYS_PER_YEAR,
            vy = 7.69901118419740425e-03 * DAYS_PER_YEAR,
            vz = -6.90460016972063023e-05 * DAYS_PER_YEAR,
          },
        }
        
        local function energy(bodies, nbody)
            local e = 0
            for i=1,#bodies do
                local bi = bodies[i]
                local vx,vy,vz,bim = bi.vx,bi.vy,bi.vz,bi.mass
                e=sqrt(vx*vx+vy*vy+vz*vz)
            end
            return e
        end

        local nbody = #bodies

        print( energy(bodies, nbody))
    ",
    ) {
        let msg = lua::to_string(&mut state, -1).unwrap();
        _ = writeln!(state.stderr, "{}", msg);
    }
}
