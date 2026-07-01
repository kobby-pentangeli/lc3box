; Counts how many times a character occurs in a string and prints the count.
; Here it counts 's' in "mississippi sassafras", so the program prints 8.

.ORIG x3000
        AND R0, R0, #0        ; R0 = running count
        LEA R1, TEXT          ; R1 -> the string
        LD  R2, TARGET
        NOT R2, R2
        ADD R2, R2, #1        ; R2 = -target, to compare by subtraction
SCAN    LDR R3, R1, #0        ; R3 = current character
        BRz  REPORT           ; the NUL terminator ends the scan
        ADD R4, R3, R2        ; zero exactly when the character matches
        BRnp NEXT
        ADD R0, R0, #1
NEXT    ADD R1, R1, #1
        BR  SCAN
REPORT  JSR PRINT_DEC
        HALT

TARGET  .FILL x0073           ; 's'
TEXT    .STRINGZ "mississippi sassafras"

; ---------------------------------------------------------------------------
; PRINT_DEC: writes the signed value in R0 to the console as decimal, followed
;            by a newline. Clobbers R0-R5; R7 is saved across the trap calls so
;            the routine is safe to reach with JSR on a conforming LC-3.
; ---------------------------------------------------------------------------
PRINT_DEC
        ST   R7, PD_R7
        ADD  R0, R0, #0       ; a negative value prints a leading '-'
        BRzp PD_MAG
        ADD  R3, R0, #0
        LD   R0, PD_MINUS
        TRAP x21
        NOT  R0, R3
        ADD  R0, R0, #1       ; R0 = magnitude
PD_MAG  LEA  R1, PD_POW       ; walk the powers of ten, most significant first
        AND  R4, R4, #0       ; R4 stays 0 while leading zeros are suppressed
PD_PLACE
        LDR  R2, R1, #0
        BRz  PD_END           ; the table ends with a zero word
        AND  R3, R3, #0       ; R3 counts how many times the power fits
PD_SUB  NOT  R5, R2
        ADD  R5, R5, #1
        ADD  R0, R0, R5       ; R0 -= power
        BRn  PD_EMIT
        ADD  R3, R3, #1
        BR   PD_SUB
PD_EMIT ADD  R0, R0, R2       ; undo the overshoot; R0 keeps the remainder
        ADD  R3, R3, #0
        BRp  PD_SHOW          ; always print a nonzero digit
        ADD  R4, R4, #0
        BRp  PD_SHOW          ; and every digit once printing has begun
        ADD  R5, R2, #-1
        BRp  PD_NEXT          ; skip a leading zero unless it is the ones place
PD_SHOW ADD  R4, R4, #1
        LD   R5, PD_ZERO
        ADD  R5, R5, R3       ; ASCII digit
        ADD  R3, R0, #0       ; hold the remainder across the trap
        ADD  R0, R5, #0
        TRAP x21
        ADD  R0, R3, #0
PD_NEXT ADD  R1, R1, #1
        BR   PD_PLACE
PD_END  LD   R0, PD_NL
        TRAP x21
        LD   R7, PD_R7
        RET
PD_MINUS .FILL x002D
PD_ZERO  .FILL x0030
PD_NL    .FILL x000A
PD_R7    .BLKW #1
PD_POW   .FILL #10000
         .FILL #1000
         .FILL #100
         .FILL #10
         .FILL #1
         .FILL #0
.END
