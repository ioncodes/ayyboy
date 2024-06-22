import re

def compare_trace(ayyboy, bootrom):
    ayyboy_pattern = re.compile(r".+\[[0-9a-fA-F]+\].+\[A: \$([0-9a-fA-F]+)  F: \$([0-9a-fA-F]+)  B: \$([0-9a-fA-F]+)  C: \$([0-9a-fA-F]+)  D: \$([0-9a-fA-F]+)  E: \$([0-9a-fA-F]+)  H: \$([0-9a-fA-F]+)  L: \$([0-9a-fA-F]+)  SP: \$([0-9a-fA-F]+)  PC: \$([0-9a-fA-F]+)")
    bootrom_pattern = re.compile(r"A: ([0-9a-fA-F]+) F: ([0-9a-fA-F]+) B: ([0-9a-fA-F]+) C: ([0-9a-fA-F]+) D: ([0-9a-fA-F]+) E: ([0-9a-fA-F]+) H: ([0-9a-fA-F]+) L: ([0-9a-fA-F]+) SP: ([0-9a-fA-F]+) PC: [0-9a-fA-F]+:([\d]+)")

    for _, (line1, line2) in enumerate(zip(ayyboy, bootrom)):
        line1 = line1.strip().upper()
        line2 = line2.strip().upper()

        try:
            match_ayyboy = ayyboy_pattern.match(line1)
            match_bootrom = bootrom_pattern.match(line2)

            if match_ayyboy.group(1) != match_bootrom.group(1):
                print(f"Error on line {line1} and {line2}")
                break
            if match_ayyboy.group(2) != match_bootrom.group(2):
                print(f"Error on line {line1} and {line2}")
                break
            if match_ayyboy.group(3) != match_bootrom.group(3):
                print(f"Error on line {line1} and {line2}")
                break
            if match_ayyboy.group(4) != match_bootrom.group(4):
                print(f"Error on line {line1} and {line2}")
                break
            if match_ayyboy.group(5) != match_bootrom.group(5):
                print(f"Error on line {line1} and {line2}")
                break
            if match_ayyboy.group(6) != match_bootrom.group(6):
                print(f"Error on line {line1} and {line2}")
                break
            if match_ayyboy.group(7) != match_bootrom.group(7):
                print(f"Error on line {line1} and {line2}")
                break
            if match_ayyboy.group(8) != match_bootrom.group(8):
                print(f"Error on line {line1} and {line2}")
                break
            if match_ayyboy.group(9) != match_bootrom.group(9):
                print(f"Error on line {line1} and {line2}")
                break
        except:
            print(f"Exception for line {line1} and {line2}")
            break

with open("ayyboy_trace.log") as f:
    ayyboy_trace = f.readlines()
with open("BootromLog.txt") as f:
    bootrom_log = f.readlines()

compare_trace(ayyboy_trace, bootrom_log)