import sys


TO_IGNORE = [
    'initGEOS_r', # deprecated in 3.5
    'finishGEOS_r',  # deprecated in 3.5
    'GEOSGeomFromWKT', # deprecated
    'GEOSGeomFromWKT_r', # deprecated 
    'GEOSGeomToWKT', # deprecated
    'GEOSGeomToWKT_r', # deprecated
    'GEOSSingleSidedBuffer', # deprecated in 3.3
    'GEOSSingleSidedBuffer_r', # deprecated in 3.3
    'GEOSUnionCascaded', # deprecated in 3.3
    'GEOSUnionCascaded_r', # deprecated in 3.3
]


def get_c_args(text, args):
    for part in text.split(','):
        part = part.strip()
        if len(part) == 0:
            continue
        params = part.split(' ')
        ty_name = params.pop()
        params = ' '.join(params)
        while ty_name.endswith('[]'):
            params += ' *const'
            ty_name = ty_name[:-2].strip()
        if len(ty_name) == 0:
            ty_name = params.pop()
        while ty_name.startswith('*'):
            params += '*'
            ty_name = ty_name[1:]
        args.append(params.replace(' * const', ' *const').strip())


def get_rust_args(text, args):
    for part in text.split(','):
        part = part.strip()
        if len(part) == 0:
            continue
        params = part.split(':')
        params.pop(0)
        args.append(' '.join([x.strip() for x in params if len(x.strip()) > 0]))


def get_c_functions(filename):
    content = None
    functions = {}
    with open(filename) as f:
        content = f.read().splitlines()
    if content is None:
        return functions
    pos = 0
    while pos < len(content):
        line = content[pos]
        if ' GEOS_DLL ' not in line:
            pos += 1
            continue
        parts = line.split(' GEOS_DLL ')
        ret_type = parts[0].split('extern ')[1].strip()
        name_and_args = parts[1].split('(')
        func_name = name_and_args[0]
        while func_name.startswith('*'):
            ret_type += '*'
            func_name = func_name[1:]
        args = []
        get_c_args(name_and_args[1], args)
        if not name_and_args[1].endswith(');'):
            pos += 1
            while pos < len(content):
                get_c_args(content[pos].strip().split(');')[0], args)
                if content[pos].endswith(');'):
                    break
                pos += 1
        func_name = func_name.strip()
        if len(func_name) > 0 and func_name not in TO_IGNORE:
            functions[func_name] = (args, ret_type)
        pos += 1
    return functions


def get_rust_ret_type(text):
    if ') -> ' not in text:
        return ''
    return text.split(') -> ')[1].split(';')[0]


def convert_c_to_rust(c_type):
    inline_conv = {
        'int': 'c_int',
        'char': 'c_char',
        'void': 'c_void',
        'double': 'c_double',
        'float': 'c_float'
    }
    unsigned_inline_conv = {
        'int': 'c_uint',
        'char': 'c_uchar',
        'c_int': 'c_uint',
        'c_char': 'c_uchar'
    }
    parts = c_type.strip().split(' ')
    is_const = False
    pos = 0
    complete_type = []
    while pos < len(parts):
        if parts[pos] == 'const':
            if pos == 0:
                is_const = True
        else:
            complete_type.append(parts[pos])
        pos += 1
    while complete_type[len(complete_type) - 1].endswith('*'):
        complete_type[len(complete_type) - 1] = complete_type[len(complete_type) - 1][:-1].strip()
        complete_type.insert(0, '*const' if is_const else '*mut')
    while complete_type[-1].endswith('*const'):
        complete_type[-1] = complete_type[-1][:-6].strip()
        if len(complete_type[-1]) == 0:
            complete_type.pop()
        complete_type.insert(0, '*const')
    pos = 0
    while pos < len(complete_type):
        if complete_type[pos] == 'unsigned':
            complete_type.pop(pos)
            complete_type[pos] = unsigned_inline_conv[complete_type[pos]]
        else:
            complete_type[pos] = inline_conv.get(complete_type[pos], complete_type[pos])
        pos += 1
    return ' '.join([x.strip() for x in complete_type if len(x.strip()) > 0])


def compare_rust_and_c_funcs(name, c_func, rust_args, ret_type):
    errors = 0
    c_func_args = ','.join([convert_c_to_rust(x) for x in c_func[0]])
    c_func_ret = convert_c_to_rust(c_func[1])
    if c_func_ret == 'c_void':
        c_func_ret = ''
    if c_func_ret != ret_type:
        print('[{}]: ret types differ: `{}` != `{}`'.format(name, c_func_ret, ret_type))
        errors += 1
    if c_func_args != ','.join(rust_args):
        print('[{}]: params differ:\n=> `{}`\n-> `{}`'.format(name, c_func_args, ','.join(rust_args)))
        errors += 1
    return errors == 0


def get_rust_functions(filename, c_func):
    errors = 0
    content = None
    functions = set()
    with open(filename) as f:
        content = f.read().splitlines()
    if content is None:
        return
    pos = 0
    while pos < len(content):
        line = content[pos]
        if ' fn ' not in line:
            pos += 1
            continue
        parts = line.split(' fn ')[1].split('(')
        func_name = parts[0].strip()
        if func_name in c_func:
            args = []
            get_rust_args(parts[1].split(')')[0].strip(), args)
            if not line.endswith(');') and not ') -> ' in line:
                pos += 1
                while pos < len(content):
                    get_rust_args(content[pos].strip().split(')')[0], args)
                    if content[pos].endswith(');') or ') -> ' in content[pos]:
                        break
                    pos += 1
            ret_type = get_rust_ret_type(content[pos])
            if not compare_rust_and_c_funcs(func_name, c_func[func_name], args, ret_type):
                errors += 1
            del c_func[func_name]
        elif len(func_name) > 0:
            functions.add(func_name)
        pos += 1
    return (functions, errors != 0)


c_func = get_c_functions('check_missing/geos_c.h')
r_func, errored = get_rust_functions('src/functions.rs', c_func)


x = []
print('==> Not bound functions:')
for f in c_func:
    x.append(f)
x.sort()
for f in x:
    print(f)
print('')
print('==> Extra (???) rust functions:')
x = []
for f in r_func:
    x.append(f)
x.sort()
for f in x:
    print(f)

if errored or len(c_func) > 0 or len(r_func) > 0:
    sys.exit(1)
