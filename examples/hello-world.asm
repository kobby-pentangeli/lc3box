; Prints "Hello World!" to the console and halts.
.ORIG x3000
        LEA R0, HELLO
        PUTS
        HALT
HELLO   .STRINGZ "Hello World!"
.END
