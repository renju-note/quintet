import re
import sys

raw = sys.stdin.readline()
m = re.findall(r"[a-oA-O][0-9]+", raw)
print(",".join(m))
