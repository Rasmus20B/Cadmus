#!/usr/bin/python3

# Used to generate Enum for scripting engine's instructions
# Uses "fmp_format.md" to construct enum

lookup = {}

with open(r"fmp_format.md", 'r') as fp:
    ins = False
    for l_no, line in enumerate(fp):
        if ins is True and not line.startswith("#### "):
            continue
        if ins is True and line.startswith("### "):
            break
        if ins is True:
            print(line)
            num = int(line.split(" ")[1][:-1])
            name = line.split(".")[1][1:].strip().title().replace(" ", "").replace("-", "").replace("/", "")
            lookup[num] = name
        if line.startswith("### List of Instructions"):
            ins = True

with open("src/decompile/script_engine_instructions.rs", 'w') as fp:
    fp.write("pub enum Instructions {\n")
    for ins in sorted(lookup.keys()):
        fp.write(f"\t{lookup[ins]} = {ins},\n")
    fp.write("}")
