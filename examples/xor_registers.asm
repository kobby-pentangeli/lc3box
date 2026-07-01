; Computes R3 = R1 XOR R2 using only AND and NOT, then prints R3 as hex.
; With R1 = xFF00 and R2 = x0FF0 the program prints F0F0.

.ORIG x3000
xor   LD  R1, VAL1
      LD  R2, VAL2
      NOT R1,R1
      AND R3,R1,R2
      NOT R1,R1
      NOT R2,R2
      AND R4,R1,R2
      NOT R2,R2
      NOT R3,R3
      NOT R4,R4
      AND R3,R3,R4
      NOT R3,R3            ; R3 = R1 XOR R2
      ADD R0,R3,#0
      JSR PRINT_HEX
      HALT

VAL1  .FILL xFF00
VAL2  .FILL x0FF0

; ---------------------------------------------------------------------------
; PRINT_HEX: writes R0 to the console as four hexadecimal digits, followed by a
;            newline. Clobbers R0-R5; R7 is saved across the trap calls so the
;            routine is safe to reach with JSR on a conforming LC-3.
; ---------------------------------------------------------------------------
PRINT_HEX
        ST   R7, PH_R7
        ADD  R4, R0, #0       ; R4 holds the word being shifted left
        AND  R3, R3, #0
        ADD  R3, R3, #4       ; four nibbles, most significant first
PH_DIG  AND  R1, R1, #0       ; R1 accumulates the current nibble
        AND  R2, R2, #0
        ADD  R2, R2, #4       ; four bits per nibble
PH_BIT  ADD  R1, R1, R1       ; nibble <<= 1
        ADD  R4, R4, #0       ; inspect the top bit of the word
        BRzp PH_ZERO
        ADD  R1, R1, #1
PH_ZERO ADD  R4, R4, R4       ; word <<= 1, discarding the bit just read
        ADD  R2, R2, #-1
        BRp  PH_BIT
        LD   R5, PH_ZEROC     ; 0-9 map to '0'..'9'
        ADD  R5, R5, R1
        ADD  R1, R1, #-10
        BRn  PH_OUT
        LD   R5, PH_ALPHA     ; 10-15 map to 'A'..'F'
        ADD  R5, R5, R1
PH_OUT  ADD  R0, R5, #0
        TRAP x21
        ADD  R3, R3, #-1
        BRp  PH_DIG
        LD   R0, PH_NL
        TRAP x21
        LD   R7, PH_R7
        RET
PH_ZEROC .FILL x0030
PH_ALPHA .FILL x0041
PH_NL    .FILL x000A
PH_R7    .BLKW #1
.END
