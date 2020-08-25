IN_DIR = "input_consts_config.txt"
OUT_DIR = "src/input/input_consts.rs"

f = open(IN_DIR, 'r')
lines = []
for line in f.readlines():
    if line != "\n":
        lines.append(line.rstrip('\n'))
f.close()

regularConsts = True
namePrefix = ""
namePostfix = ""
valuePrefix = ""
valuePostfix = ""
_type = ""
output = "#![allow(dead_code)]\n#![allow(non_upper_case_globals)]\n"


def fmt(name, value):
    return f"pub const {namePrefix + name + namePostfix}: {_type} = {valuePrefix + value + valuePostfix};\n"


for line in lines:
    if line == "Regular:":
        regularConsts = True
    elif line == "Named:":
        regularConsts = False
    elif line[0:11] == "NamePrefix:":
        namePrefix = line[11:]
    elif line[0:12] == "NamePostfix:":
        namePostfix = line[12:]
    elif line[0:12] == "ValuePrefix:":
        valuePrefix = line[12:]
    elif line[0:13] == "ValuePostfix:":
        valuePostfix = line[13:]
    elif line[0:5] == "Type:":
        _type = line[5:]
    else:
        if regularConsts:
            output += fmt(line, line)
        else:
            parts = []
            if line[-1] == ',':
                parts = [line.split(',')[0], ","]
            else:
                parts = line.split(',')
            output += fmt(parts[0], parts[1])

with open(OUT_DIR, 'w') as f:
    f.write(output)
