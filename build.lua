#!/bin/lua
function GetOsName()
    local raw_os_name = '';
    if package.config:sub(1, 1) == '\\' then
        raw_os_name = "windows";
    else
        raw_os_name = io.popen('uname -s', 'r'):read('*l');
    end
    raw_os_name = (raw_os_name):lower();
    local os_patterns = {
        ['windows'] = 'windows',
        ['linux']   = 'linux',
        ['osx']     = 'macos',
        ['mac']     = 'macos',
        ['darwin']  = 'macos',
        ['^mingw']  = 'windows',
        ['^cygwin'] = 'windows',
    };
    local os_name = 'unknown';
    for pattern, name in pairs(os_patterns) do
        if raw_os_name:match(pattern) then
            os_name = name;
            break;
        end
    end
    return os_name;
end

function IsInArray(value, array)
    for _, v in ipairs(array) do
        if v == value then
            return true;
        end
    end
    return false;
end

function B2N(b)
    if b then
        return 1
    else
        return 0
    end
end

function ConstantsGen()
    local function trim(s)
        return s:match "^%s*(.-)%s*$"
    end
    local js_code = [[
export const IS_DEBUG: boolean = %s;
export const OS_NAME: string = "%s";
export const CAN_USE_CAM: boolean = %s;
export const CAN_USE_CAP: boolean = %s;
export const GET_QR_CODE_TYPE_COUNT: number = %d;
    ]];
    js_code = string.format(js_code, IS_DEBUG, OS_NAME, CAN_USE_CAM, CAN_USE_CAP, GET_QR_CODE_TYPE_COUNT);
    js_code = trim(js_code);
    local file = io.open("./src/lib/commands/constants.ts", "w");
    if file then
        file:write(js_code);
        file:close()
    end
end

function GetCmd()
    local prefix = "pnpm tauri";
    local cmd = prefix .. " ";
    if IS_MOBILE then
        cmd = cmd .. OS_NAME .. " build ";
    else
        cmd = cmd .. "build ";
    end
    if IS_DEBUG then
        cmd = cmd .. "--debug";
    end
    local args = { table.unpack(arg, UNPACK_INDEX) };
    cmd = cmd .. " " .. table.concat(args, " ");
    return cmd;
end

function Arg2Profile(arg_)
    if IsInArray(arg_, __RELEASE) then
        return "release";
    end
    if IsInArray(arg_, __DEBUG) then
        return "debug";
    end
    return arg_
end

__MOBILE = { "android", "ios" };
__DESKTOP = { "windows", "linux", "macos" };
__PROFILE = { "debug", "release" };
__RELEASE = { '-r', "--release", "r", "release" };
__DEBUG = { "--debug", "debug" };

if arg[1] == "h" or arg[1] == "help" or arg[1] == "-h" or arg[1] == "--help" then
    print("用法：build.lua [OS_NAME] [PROFILE] [...ARGS]");
    print("    [OS_NAME] 支持的值有：");
    print("        MOBILE:  ", table.concat(__MOBILE, ", "));
    print("        DESKTOP: ", table.concat(__DESKTOP, ", "));
    print("    [PROFILE] 支持的值有：");
    print("        RELEASE: ", table.concat(__RELEASE, ", "));
    print("        DEBUG:   ", table.concat(__DEBUG, ", "));
    print("打印本信息：build.lua [-h / h / --help / help]");
    return;
end

OS_NAME = arg[1];
PROFILE = Arg2Profile(arg[2]);
UNPACK_INDEX = 3;

IS_MOBILE = false;
IS_DESKTOP = false;

IS_MOBILE = IsInArray(OS_NAME, __MOBILE);
IS_DESKTOP = IsInArray(OS_NAME, __DESKTOP);
LOCAL_OS_NAME = GetOsName();

if not IS_MOBILE and not IS_DESKTOP then
    if LOCAL_OS_NAME ~= "unknown" then
        OS_NAME = LOCAL_OS_NAME
        IS_MOBILE = IsInArray(LOCAL_OS_NAME, __MOBILE);
        IS_DESKTOP = IsInArray(LOCAL_OS_NAME, __DESKTOP);
        PROFILE = Arg2Profile(arg[1]);
        UNPACK_INDEX = UNPACK_INDEX - 1;
    else
        print("当前环境不支持推断编译目标，请指定 <OS_NAME>.");
        print("<OS_NAME> 支持的值有：");
        print("MOBILE: ", table.concat(__MOBILE, ", "));
        print("DESKTOP: ", table.concat(__DESKTOP, ", "));
        return 1;
    end
end
if not IsInArray(PROFILE, __PROFILE) then
    PROFILE = "debug"
    UNPACK_INDEX = UNPACK_INDEX - 1;
end
IS_DEBUG = PROFILE == "debug"
CAN_USE_CAM = IsInArray(OS_NAME, { "android", "ios" });
CAN_USE_CAP = IsInArray(OS_NAME, { "windows", "macos", "linux" });
GET_QR_CODE_TYPE_COUNT = B2N(CAN_USE_CAM) + B2N(CAN_USE_CAP);

ConstantsGen();
CMD = GetCmd();
os.execute(CMD);
