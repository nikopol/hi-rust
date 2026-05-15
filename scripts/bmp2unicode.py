#21212
bmp = """
*
*
*
*
------
*
*
*    *
*    *
------
*    *
 *   *
  ****
     *
------
     *
     *
     *
     *
------
     *
     *
     *
     *
"""

# Braille 2x4 bit layout (Unicode U+2800):
# col 0: rows 0-2 → bits 0,1,2 / row 3 → bit 6
# col 1: rows 0-2 → bits 3,4,5 / row 3 → bit 7
BRAILLE_BITS = [
    [0, 3],
    [1, 4],
    [2, 5],
    [6, 7],
]

def segment_to_braille(rows: list[str]) -> str:
    width = max((len(r) for r in rows), default=0)
    ncells = (width + 1) // 2
    result = []
    for cell in range(ncells):
        bits = 0
        for r, row in enumerate(rows[:4]):
            for c in range(2):
                col = cell * 2 + c
                if col < len(row) and row[col] != ' ' and row[col] != '.':
                    bits |= 1 << BRAILLE_BITS[r][c]
        result.append(chr(0x2800 + bits))
    return ''.join(result)

def bmp_to_braille(bmp: str) -> str:
    segment: list[str] = []
    output: list[str] = []
    for line in bmp.splitlines():
        if all(c == '-' for c in line) and line:
            if segment:
                output.append(segment_to_braille(segment))
                segment = []
        elif line:
            segment.append(line)
    if segment:
        output.append(segment_to_braille(segment))
    return '\n'.join(output)

print(bmp_to_braille(bmp))
