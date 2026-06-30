; ======================================================================
; 2048 – The classic sliding puzzle.
; ======================================================================
; The board is 4x4.  Tiles are powers of two (2, 4, 8, …).
; After every move, a new tile (2 or 4) spawns in a random empty cell.
;
; Objective: Slide all tiles in one direction; identical tiles that collide
; merge into their sum.  Reach the 2048 tile to win!
; The game ends when no more moves are possible.
;
; Controls (press the key, no Enter needed):
;   W / A / S / D   –  slide up / left / down / right
;   Ctrl+C          –  quit
;
; How high can you go?
; ======================================================================

.ORIG x3000
        LD R6, L_3015           ; x3000 x2C14
        LEA R5, L_3017          ; x3001 xEA15
        LEA R0, L_3080          ; x3002 xE07D
        PUTS                    ; x3003 xF022
        LEA R0, L_3028          ; x3004 xE023
        JSR L_32D4              ; x3005 x4ACE
        BRp L_3008              ; x3006 x0201
        STI R0, L_3027          ; x3007 xB01F
L_3008  JSR L_30A3              ; x3008 x489A
L_3009  JSR L_31E5              ; x3009 x49DB
        JSR L_30B2              ; x300A x48A7
        LD R0, L_3016           ; x300B x200A
        BRp L_300E              ; x300C x0201
        BR L_3009               ; x300D x0FFB
L_300E  JSR L_31E5              ; x300E x49D6
        LEA R0, L_3071          ; x300F xE061
        PUTS                    ; x3010 xF022
        LEA R0, L_304C          ; x3011 xE03A
        JSR L_32D4              ; x3012 x4AC1
        BRp L_3008              ; x3013 x03F4
        HALT                    ; x3014 xF025
L_3015  JSRR R0                 ; x3015 x4000
L_3016  .FILL x0000             ; x3016 x0000
L_3017  .FILL x0001             ; x3017 x0001
        .FILL x0007             ; x3018 x0007
        .FILL x0008             ; x3019 x0008
        .FILL x000F             ; x301A x000F
        .FILL x0001             ; x301B x0001
        .FILL x0006             ; x301C x0006
        .FILL x0009             ; x301D x0009
        .FILL x000E             ; x301E x000E
        .FILL x0002             ; x301F x0002
        .FILL x0005             ; x3020 x0005
        .FILL x000A             ; x3021 x000A
        .FILL x000D             ; x3022 x000D
        .FILL x0003             ; x3023 x0003
        .FILL x0004             ; x3024 x0004
        .FILL x000B             ; x3025 x000B
        .FILL x000C             ; x3026 x000C
L_3027  ST R1, L_3041           ; x3027 x3219
L_3028  .FILL x0041             ; x3028 x0041
        .FILL x0072             ; x3029 x0072
        .FILL x0065             ; x302A x0065
        .FILL x0020             ; x302B x0020
        .FILL x0079             ; x302C x0079
        .FILL x006F             ; x302D x006F
        .FILL x0075             ; x302E x0075
        .FILL x0020             ; x302F x0020
        .FILL x006F             ; x3030 x006F
        .FILL x006E             ; x3031 x006E
        .FILL x0020             ; x3032 x0020
        .FILL x0061             ; x3033 x0061
        .FILL x006E             ; x3034 x006E
        .FILL x0020             ; x3035 x0020
        .FILL x0041             ; x3036 x0041
        .FILL x004E             ; x3037 x004E
        .FILL x0053             ; x3038 x0053
        .FILL x0049             ; x3039 x0049
        .FILL x0020             ; x303A x0020
        .FILL x0074             ; x303B x0074
        .FILL x0065             ; x303C x0065
        .FILL x0072             ; x303D x0072
        .FILL x006D             ; x303E x006D
        .FILL x0069             ; x303F x0069
        .FILL x006E             ; x3040 x006E
L_3041  .FILL x0061             ; x3041 x0061
        .FILL x006C             ; x3042 x006C
        .FILL x0020             ; x3043 x0020
        .FILL x0028             ; x3044 x0028
        .FILL x0079             ; x3045 x0079
        .FILL x002F             ; x3046 x002F
        .FILL x006E             ; x3047 x006E
        .FILL x0029             ; x3048 x0029
        .FILL x003F             ; x3049 x003F
        .FILL x0020             ; x304A x0020
        .FILL x0000             ; x304B x0000
L_304C  .FILL x0057             ; x304C x0057
        .FILL x006F             ; x304D x006F
        .FILL x0075             ; x304E x0075
        .FILL x006C             ; x304F x006C
        .FILL x0064             ; x3050 x0064
        .FILL x0020             ; x3051 x0020
        .FILL x0079             ; x3052 x0079
        .FILL x006F             ; x3053 x006F
        .FILL x0075             ; x3054 x0075
        .FILL x0020             ; x3055 x0020
        .FILL x006C             ; x3056 x006C
        .FILL x0069             ; x3057 x0069
        .FILL x006B             ; x3058 x006B
        .FILL x0065             ; x3059 x0065
        .FILL x0020             ; x305A x0020
        .FILL x0074             ; x305B x0074
        .FILL x006F             ; x305C x006F
        .FILL x0020             ; x305D x0020
        .FILL x0070             ; x305E x0070
        .FILL x006C             ; x305F x006C
        .FILL x0061             ; x3060 x0061
        .FILL x0079             ; x3061 x0079
        .FILL x0020             ; x3062 x0020
        .FILL x0061             ; x3063 x0061
        .FILL x0067             ; x3064 x0067
        .FILL x0061             ; x3065 x0061
        .FILL x0069             ; x3066 x0069
        .FILL x006E             ; x3067 x006E
        .FILL x0020             ; x3068 x0020
        .FILL x0028             ; x3069 x0028
        .FILL x0079             ; x306A x0079
        .FILL x002F             ; x306B x002F
        .FILL x006E             ; x306C x006E
        .FILL x0029             ; x306D x0029
        .FILL x003F             ; x306E x003F
        .FILL x0020             ; x306F x0020
        .FILL x0000             ; x3070 x0000
L_3071  .FILL x000A             ; x3071 x000A
        .FILL x0059             ; x3072 x0059
        .FILL x006F             ; x3073 x006F
        .FILL x0075             ; x3074 x0075
        .FILL x0020             ; x3075 x0020
        .FILL x006C             ; x3076 x006C
        .FILL x006F             ; x3077 x006F
        .FILL x0073             ; x3078 x0073
        .FILL x0074             ; x3079 x0074
        .FILL x0020             ; x307A x0020
        .FILL x003A             ; x307B x003A
        .FILL x0028             ; x307C x0028
        .FILL x000A             ; x307D x000A
        .FILL x000A             ; x307E x000A
        .FILL x0000             ; x307F x0000
L_3080  .FILL x0043             ; x3080 x0043
        .FILL x006F             ; x3081 x006F
        .FILL x006E             ; x3082 x006E
        .FILL x0074             ; x3083 x0074
        .FILL x0072             ; x3084 x0072
        .FILL x006F             ; x3085 x006F
        .FILL x006C             ; x3086 x006C
        .FILL x0020             ; x3087 x0020
        .FILL x0074             ; x3088 x0074
        .FILL x0068             ; x3089 x0068
        .FILL x0065             ; x308A x0065
        .FILL x0020             ; x308B x0020
        .FILL x0067             ; x308C x0067
        .FILL x0061             ; x308D x0061
        .FILL x006D             ; x308E x006D
        .FILL x0065             ; x308F x0065
        .FILL x0020             ; x3090 x0020
        .FILL x0075             ; x3091 x0075
        .FILL x0073             ; x3092 x0073
        .FILL x0069             ; x3093 x0069
        .FILL x006E             ; x3094 x006E
        .FILL x0067             ; x3095 x0067
        .FILL x0020             ; x3096 x0020
        .FILL x0057             ; x3097 x0057
        .FILL x0041             ; x3098 x0041
        .FILL x0053             ; x3099 x0053
        .FILL x0044             ; x309A x0044
        .FILL x0020             ; x309B x0020
        .FILL x006B             ; x309C x006B
        .FILL x0065             ; x309D x0065
        .FILL x0079             ; x309E x0079
        .FILL x0073             ; x309F x0073
        .FILL x002E             ; x30A0 x002E
        .FILL x000A             ; x30A1 x000A
        .FILL x0000             ; x30A2 x0000
L_30A3  STR R7, R6, #-1         ; x30A3 x7FBF
        ADD R6, R6, #-1         ; x30A4 x1DBF
        AND R0, R0, #0          ; x30A5 x5020
        AND R1, R1, #0          ; x30A6 x5260
        ST R0, L_3016           ; x30A7 x316E
L_30A8  ADD R2, R1, R5          ; x30A8 x1445
        STR R0, R2, #0          ; x30A9 x7080
        ADD R1, R1, #1          ; x30AA x1261
        ADD R2, R1, #-16        ; x30AB x1470
        BRn L_30A8              ; x30AC x09FB
        JSR L_3196              ; x30AD x48E8
        JSR L_3196              ; x30AE x48E7
        LDR R7, R6, #0          ; x30AF x6F80
        ADD R6, R6, #1          ; x30B0 x1DA1
        RET                     ; x30B1 xC1C0
L_30B2  STR R7, R6, #-1         ; x30B2 x7FBF
        ADD R6, R6, #-1         ; x30B3 x1DBF
L_30B4  GETC                    ; x30B4 xF020
        LD R1, L_30DF           ; x30B5 x2229
        ADD R1, R0, R1          ; x30B6 x1201
        BRz L_30CA              ; x30B7 x0412
        LD R1, L_30E0           ; x30B8 x2227
        ADD R1, R0, R1          ; x30B9 x1201
        BRz L_30C2              ; x30BA x0407
        LD R1, L_30E1           ; x30BB x2225
        ADD R1, R0, R1          ; x30BC x1201
        BRz L_30D0              ; x30BD x0412
        LD R1, L_30E2           ; x30BE x2223
        ADD R1, R0, R1          ; x30BF x1201
        BRz L_30C4              ; x30C0 x0403
        BR L_30B4               ; x30C1 x0FF2
L_30C2  JSR L_3114              ; x30C2 x4851
        BR L_30D5               ; x30C3 x0E11
L_30C4  JSR L_30E3              ; x30C4 x481E
        JSR L_30E3              ; x30C5 x481D
        JSR L_3114              ; x30C6 x484D
        JSR L_30E3              ; x30C7 x481B
        JSR L_30E3              ; x30C8 x481A
        BR L_30D5               ; x30C9 x0E0B
L_30CA  JSR L_30E3              ; x30CA x4818
        JSR L_30E3              ; x30CB x4817
        JSR L_30E3              ; x30CC x4816
        JSR L_3114              ; x30CD x4846
        JSR L_30E3              ; x30CE x4814
        BR L_30D5               ; x30CF x0E05
L_30D0  JSR L_30E3              ; x30D0 x4812
        JSR L_3114              ; x30D1 x4842
        JSR L_30E3              ; x30D2 x4810
        JSR L_30E3              ; x30D3 x480F
        JSR L_30E3              ; x30D4 x480E
L_30D5  ADD R0, R0, #0          ; x30D5 x1020
        BRnz L_30B4             ; x30D6 x0DDD
        JSR L_3196              ; x30D7 x48BE
        ADD R0, R0, #0          ; x30D8 x1020
        BRp L_30DC              ; x30D9 x0202
        JSR L_31B7              ; x30DA x48DC
        ST R0, L_3016           ; x30DB x313A
L_30DC  LDR R7, R6, #0          ; x30DC x6F80
        ADD R6, R6, #1          ; x30DD x1DA1
        RET                     ; x30DE xC1C0
L_30DF  .FILL xFF89             ; x30DF xFF89
L_30E0  .FILL xFF9F             ; x30E0 xFF9F
L_30E1  .FILL xFF8D             ; x30E1 xFF8D
L_30E2  .FILL xFF9C             ; x30E2 xFF9C
L_30E3  STR R0, R6, #-1         ; x30E3 x71BF
        ADD R6, R6, #-1         ; x30E4 x1DBF
        LDR R0, R5, #1          ; x30E5 x6141
        STR R0, R6, #-1         ; x30E6 x71BF
        LDR R0, R5, #2          ; x30E7 x6142
        STR R0, R6, #-2         ; x30E8 x71BE
        LDR R0, R5, #3          ; x30E9 x6143
        STR R0, R6, #-3         ; x30EA x71BD
        ADD R6, R6, #-3         ; x30EB x1DBD
        LDR R0, R5, #0          ; x30EC x6140
        STR R0, R5, #3          ; x30ED x7143
        LDR R0, R5, #4          ; x30EE x6144
        STR R0, R5, #2          ; x30EF x7142
        LDR R0, R5, #8          ; x30F0 x6148
        STR R0, R5, #1          ; x30F1 x7141
        LDR R0, R5, #12         ; x30F2 x614C
        STR R0, R5, #0          ; x30F3 x7140
        LDR R0, R5, #13         ; x30F4 x614D
        STR R0, R5, #4          ; x30F5 x7144
        LDR R0, R5, #14         ; x30F6 x614E
        STR R0, R5, #8          ; x30F7 x7148
        LDR R0, R5, #15         ; x30F8 x614F
        STR R0, R5, #12         ; x30F9 x714C
        LDR R0, R5, #11         ; x30FA x614B
        STR R0, R5, #13         ; x30FB x714D
        LDR R0, R5, #7          ; x30FC x6147
        STR R0, R5, #14         ; x30FD x714E
        LDR R0, R6, #0          ; x30FE x6180
        STR R0, R5, #15         ; x30FF x714F
        LDR R0, R6, #1          ; x3100 x6181
        STR R0, R5, #11         ; x3101 x714B
        LDR R0, R6, #2          ; x3102 x6182
        STR R0, R5, #7          ; x3103 x7147
        ADD R6, R6, #3          ; x3104 x1DA3
        LDR R0, R5, #6          ; x3105 x6146
        STR R0, R6, #-1         ; x3106 x71BF
        ADD R6, R6, #-1         ; x3107 x1DBF
        LDR R0, R5, #5          ; x3108 x6145
        STR R0, R5, #6          ; x3109 x7146
        LDR R0, R5, #9          ; x310A x6149
        STR R0, R5, #5          ; x310B x7145
        LDR R0, R5, #10         ; x310C x614A
        STR R0, R5, #9          ; x310D x7149
        LDR R0, R6, #0          ; x310E x6180
        STR R0, R5, #10         ; x310F x714A
        ADD R6, R6, #1          ; x3110 x1DA1
        LDR R0, R6, #0          ; x3111 x6180
        ADD R6, R6, #1          ; x3112 x1DA1
        RET                     ; x3113 xC1C0
L_3114  STR R7, R6, #-1         ; x3114 x7FBF
        STR R1, R6, #-2         ; x3115 x73BE
        ADD R6, R6, #-2         ; x3116 x1DBE
        AND R1, R1, #0          ; x3117 x5260
        ADD R0, R5, #0          ; x3118 x1160
        JSR L_3128              ; x3119 x480E
        ADD R1, R0, R1          ; x311A x1201
        ADD R0, R5, #4          ; x311B x1164
        JSR L_3128              ; x311C x480B
        ADD R1, R0, R1          ; x311D x1201
        ADD R0, R5, #8          ; x311E x1168
        JSR L_3128              ; x311F x4808
        ADD R1, R0, R1          ; x3120 x1201
        ADD R0, R5, #12         ; x3121 x116C
        JSR L_3128              ; x3122 x4805
        ADD R0, R0, R1          ; x3123 x1001
        LDR R1, R6, #0          ; x3124 x6380
        LDR R7, R6, #1          ; x3125 x6F81
        ADD R6, R6, #2          ; x3126 x1DA2
        RET                     ; x3127 xC1C0
L_3128  STR R1, R6, #-1         ; x3128 x73BF
        ADD R6, R6, #-1         ; x3129 x1DBF
        AND R1, R1, #0          ; x312A x5260
        AND R2, R2, #0          ; x312B x54A0
        LDR R4, R0, #0          ; x312C x6800
        ADD R3, R4, R4          ; x312D x1704
        ADD R3, R3, R3          ; x312E x16C3
        ADD R3, R3, R3          ; x312F x16C3
        ADD R3, R3, R3          ; x3130 x16C3
        LDR R4, R0, #1          ; x3131 x6801
        ADD R3, R3, R4          ; x3132 x16C4
        ADD R3, R3, R3          ; x3133 x16C3
        ADD R3, R3, R3          ; x3134 x16C3
        ADD R3, R3, R3          ; x3135 x16C3
        ADD R3, R3, R3          ; x3136 x16C3
        LDR R4, R0, #2          ; x3137 x6802
        ADD R3, R3, R4          ; x3138 x16C4
        ADD R3, R3, R3          ; x3139 x16C3
        ADD R3, R3, R3          ; x313A x16C3
        ADD R3, R3, R3          ; x313B x16C3
        ADD R3, R3, R3          ; x313C x16C3
        LDR R4, R0, #3          ; x313D x6803
        ADD R3, R3, R4          ; x313E x16C4
        STR R3, R6, #-1         ; x313F x77BF
        ADD R6, R6, #-1         ; x3140 x1DBF
L_3141  ADD R4, R0, R1          ; x3141 x1801
        LDR R4, R4, #0          ; x3142 x6900
        BRnz L_3147             ; x3143 x0C03
        ADD R3, R0, R2          ; x3144 x1602
        ADD R2, R2, #1          ; x3145 x14A1
        STR R4, R3, #0          ; x3146 x78C0
L_3147  ADD R1, R1, #1          ; x3147 x1261
        ADD R4, R1, #-4         ; x3148 x187C
        BRn L_3141              ; x3149 x09F7
        AND R1, R1, #0          ; x314A x5260
L_314B  ADD R4, R2, #-4         ; x314B x18BC
        BRz L_3151              ; x314C x0404
        ADD R3, R0, R2          ; x314D x1602
        ADD R2, R2, #1          ; x314E x14A1
        STR R1, R3, #0          ; x314F x72C0
        BR L_314B               ; x3150 x0FFA
L_3151  LDR R1, R0, #0          ; x3151 x6200
        LDR R3, R0, #1          ; x3152 x6601
        BRz L_3179              ; x3153 x0425
        NOT R3, R3              ; x3154 x96FF
        ADD R3, R3, #1          ; x3155 x16E1
        ADD R3, R1, R3          ; x3156 x1643
        BRnp L_3160             ; x3157 x0A08
        ADD R1, R1, #1          ; x3158 x1261
        STR R1, R0, #0          ; x3159 x7200
        LDR R1, R0, #2          ; x315A x6202
        STR R1, R0, #1          ; x315B x7201
        LDR R1, R0, #3          ; x315C x6203
        STR R1, R0, #2          ; x315D x7202
        AND R1, R1, #0          ; x315E x5260
        STR R1, R0, #3          ; x315F x7203
L_3160  LDR R1, R0, #1          ; x3160 x6201
        LDR R3, R0, #2          ; x3161 x6602
        BRz L_3179              ; x3162 x0416
        NOT R3, R3              ; x3163 x96FF
        ADD R3, R3, #1          ; x3164 x16E1
        ADD R3, R1, R3          ; x3165 x1643
        BRnp L_316E             ; x3166 x0A07
        ADD R1, R1, #1          ; x3167 x1261
        STR R1, R0, #1          ; x3168 x7201
        LDR R1, R0, #3          ; x3169 x6203
        STR R1, R0, #2          ; x316A x7202
        AND R1, R1, #0          ; x316B x5260
        STR R1, R0, #3          ; x316C x7203
        BR L_3179               ; x316D x0E0B
L_316E  LDR R1, R0, #2          ; x316E x6202
        LDR R3, R0, #3          ; x316F x6603
        BRz L_3179              ; x3170 x0408
        NOT R3, R3              ; x3171 x96FF
        ADD R3, R3, #1          ; x3172 x16E1
        ADD R3, R1, R3          ; x3173 x1643
        BRnp L_3179             ; x3174 x0A04
        ADD R1, R1, #1          ; x3175 x1261
        STR R1, R0, #2          ; x3176 x7202
        AND R1, R1, #0          ; x3177 x5260
        STR R1, R0, #3          ; x3178 x7203
L_3179  LDR R4, R0, #0          ; x3179 x6800
        ADD R3, R4, R4          ; x317A x1704
        ADD R3, R3, R3          ; x317B x16C3
        ADD R3, R3, R3          ; x317C x16C3
        ADD R3, R3, R3          ; x317D x16C3
        LDR R4, R0, #1          ; x317E x6801
        ADD R3, R3, R4          ; x317F x16C4
        ADD R3, R3, R3          ; x3180 x16C3
        ADD R3, R3, R3          ; x3181 x16C3
        ADD R3, R3, R3          ; x3182 x16C3
        ADD R3, R3, R3          ; x3183 x16C3
        LDR R4, R0, #2          ; x3184 x6802
        ADD R3, R3, R4          ; x3185 x16C4
        ADD R3, R3, R3          ; x3186 x16C3
        ADD R3, R3, R3          ; x3187 x16C3
        ADD R3, R3, R3          ; x3188 x16C3
        ADD R3, R3, R3          ; x3189 x16C3
        LDR R4, R0, #3          ; x318A x6803
        ADD R3, R3, R4          ; x318B x16C4
        NOT R3, R3              ; x318C x96FF
        ADD R3, R3, #1          ; x318D x16E1
        LDR R4, R6, #0          ; x318E x6980
        AND R0, R0, #0          ; x318F x5020
        ADD R3, R3, R4          ; x3190 x16C4
        BRz L_3193              ; x3191 x0401
        ADD R0, R0, #1          ; x3192 x1021
L_3193  LDR R1, R6, #1          ; x3193 x6381
        ADD R6, R6, #2          ; x3194 x1DA2
        RET                     ; x3195 xC1C0
L_3196  STR R7, R6, #-1         ; x3196 x7FBF
        ADD R6, R6, #-1         ; x3197 x1DBF
        AND R1, R1, #0          ; x3198 x5260
        AND R2, R2, #0          ; x3199 x54A0
L_319A  ADD R0, R2, R5          ; x319A x1085
        ADD R2, R2, #1          ; x319B x14A1
        STR R0, R6, #-1         ; x319C x71BF
        LDR R0, R0, #0          ; x319D x6000
        BRp L_31A1              ; x319E x0202
        ADD R1, R1, #1          ; x319F x1261
        ADD R6, R6, #-1         ; x31A0 x1DBF
L_31A1  ADD R0, R2, #-16        ; x31A1 x10B0
        BRn L_319A              ; x31A2 x09F7
        ADD R0, R1, #0          ; x31A3 x1060
        BRz L_31B2              ; x31A4 x040D
        JSR L_32BD              ; x31A5 x4917
        ADD R2, R0, R6          ; x31A6 x1406
        LD R0, L_31B6           ; x31A7 x200E
        JSR L_32BD              ; x31A8 x4914
        ADD R0, R0, #0          ; x31A9 x1020
        BRz L_31AE              ; x31AA x0403
        AND R0, R0, #0          ; x31AB x5020
        ADD R0, R0, #1          ; x31AC x1021
        BR L_31AF               ; x31AD x0E01
L_31AE  ADD R0, R0, #2          ; x31AE x1022
L_31AF  LDR R2, R2, #0          ; x31AF x6480
        STR R0, R2, #0          ; x31B0 x7080
        ADD R0, R1, #-1         ; x31B1 x107F
L_31B2  ADD R6, R6, R1          ; x31B2 x1D81
        LDR R7, R6, #0          ; x31B3 x6F80
        ADD R6, R6, #1          ; x31B4 x1DA1
        RET                     ; x31B5 xC1C0
L_31B6  .FILL x000B             ; x31B6 x000B
L_31B7  STR R7, R6, #-1         ; x31B7 x7FBF
        ADD R6, R6, #-1         ; x31B8 x1DBF
        AND R4, R4, #0          ; x31B9 x5920
        ADD R4, R4, #1          ; x31BA x1921
L_31BB  ADD R0, R5, #0          ; x31BB x1160
        JSR L_31D4              ; x31BC x4817
        BRz L_31CB              ; x31BD x040D
        ADD R0, R5, #4          ; x31BE x1164
        JSR L_31D4              ; x31BF x4814
        BRz L_31CB              ; x31C0 x040A
        ADD R0, R5, #8          ; x31C1 x1168
        JSR L_31D4              ; x31C2 x4811
        BRz L_31CB              ; x31C3 x0407
        ADD R0, R5, #12         ; x31C4 x116C
        JSR L_31D4              ; x31C5 x480E
        BRz L_31CB              ; x31C6 x0404
        ADD R4, R4, #-1         ; x31C7 x193F
        BRn L_31CB              ; x31C8 x0802
        JSR L_30E3              ; x31C9 x4F19
        BR L_31BB               ; x31CA x0FF0
L_31CB  ADD R4, R4, #0          ; x31CB x1920
        BRp L_31D0              ; x31CC x0203
        JSR L_30E3              ; x31CD x4F15
        JSR L_30E3              ; x31CE x4F14
        JSR L_30E3              ; x31CF x4F13
L_31D0  LDR R7, R6, #0          ; x31D0 x6F80
        ADD R6, R6, #1          ; x31D1 x1DA1
        ADD R0, R1, #0          ; x31D2 x1060
        RET                     ; x31D3 xC1C0
L_31D4  LDR R2, R0, #0          ; x31D4 x6400
        LDR R3, R0, #1          ; x31D5 x6601
        NOT R3, R3              ; x31D6 x96FF
        ADD R3, R3, #1          ; x31D7 x16E1
        ADD R1, R2, R3          ; x31D8 x1283
        BRz L_31E4              ; x31D9 x040A
        LDR R2, R0, #2          ; x31DA x6402
        ADD R1, R2, R3          ; x31DB x1283
        BRz L_31E4              ; x31DC x0407
        LDR R3, R0, #3          ; x31DD x6603
        NOT R3, R3              ; x31DE x96FF
        ADD R3, R3, #1          ; x31DF x16E1
        ADD R1, R2, R3          ; x31E0 x1283
        BRz L_31E4              ; x31E1 x0402
        AND R1, R1, #0          ; x31E2 x5260
        ADD R1, R1, #1          ; x31E3 x1261
L_31E4  RET                     ; x31E4 xC1C0
L_31E5  STR R7, R6, #-1         ; x31E5 x7FBF
        ADD R6, R6, #-1         ; x31E6 x1DBF
        LEA R0, L_3219          ; x31E7 xE031
        PUTS                    ; x31E8 xF022
        LEA R1, L_3268          ; x31E9 xE27E
        AND R2, R2, #0          ; x31EA x54A0
        LEA R0, L_3225          ; x31EB xE039
        PUTS                    ; x31EC xF022
        LD R0, L_3267           ; x31ED x2079
        OUT                     ; x31EE xF021
L_31EF  LEA R0, L_3242          ; x31EF xE052
        PUTS                    ; x31F0 xF022
        LD R0, L_3267           ; x31F1 x2075
        OUT                     ; x31F2 xF021
        LEA R0, L_325F          ; x31F3 xE06B
        PUTS                    ; x31F4 xF022
L_31F5  LD R0, L_3266           ; x31F5 x2070
        OUT                     ; x31F6 xF021
        ADD R3, R5, R2          ; x31F7 x1742
        LDR R3, R3, #0          ; x31F8 x66C0
        ADD R2, R2, #1          ; x31F9 x14A1
        ADD R0, R3, R3          ; x31FA x10C3
        ADD R0, R0, R0          ; x31FB x1000
        ADD R0, R0, R3          ; x31FC x1003
        ADD R0, R0, R1          ; x31FD x1001
        PUTS                    ; x31FE xF022
        LD R0, L_3266           ; x31FF x2066
        OUT                     ; x3200 xF021
        ADD R0, R2, #-4         ; x3201 x10BC
        BRz L_320A              ; x3202 x0407
        ADD R0, R2, #-8         ; x3203 x10B8
        BRz L_320A              ; x3204 x0405
        ADD R0, R2, #-12        ; x3205 x10B4
        BRz L_320A              ; x3206 x0403
        ADD R0, R2, #-16        ; x3207 x10B0
        BRz L_320A              ; x3208 x0401
        BRnp L_31F5             ; x3209 x0BEB
L_320A  LEA R0, L_3262          ; x320A xE057
        PUTS                    ; x320B xF022
        ADD R0, R2, #-16        ; x320C x10B0
        BRnp L_31EF             ; x320D x0BE1
        LEA R0, L_3242          ; x320E xE033
        PUTS                    ; x320F xF022
        LD R0, L_3267           ; x3210 x2056
        OUT                     ; x3211 xF021
        LEA R0, L_3225          ; x3212 xE012
        PUTS                    ; x3213 xF022
        LD R0, L_3267           ; x3214 x2052
        OUT                     ; x3215 xF021
        LDR R7, R6, #0          ; x3216 x6F80
        ADD R6, R6, #1          ; x3217 x1DA1
        RET                     ; x3218 xC1C0
L_3219  .FILL x001B             ; x3219 x001B
        .FILL x005B             ; x321A x005B
        .FILL x0032             ; x321B x0032
        .FILL x004A             ; x321C x004A
        .FILL x001B             ; x321D x001B
        .FILL x005B             ; x321E x005B
        .FILL x0048             ; x321F x0048
        .FILL x001B             ; x3220 x001B
        .FILL x005B             ; x3221 x005B
        .FILL x0033             ; x3222 x0033
        .FILL x004A             ; x3223 x004A
        .FILL x0000             ; x3224 x0000
L_3225  .FILL x002B             ; x3225 x002B
        .FILL x002D             ; x3226 x002D
        .FILL x002D             ; x3227 x002D
        .FILL x002D             ; x3228 x002D
        .FILL x002D             ; x3229 x002D
        .FILL x002D             ; x322A x002D
        .FILL x002D             ; x322B x002D
        .FILL x002D             ; x322C x002D
        .FILL x002D             ; x322D x002D
        .FILL x002D             ; x322E x002D
        .FILL x002D             ; x322F x002D
        .FILL x002D             ; x3230 x002D
        .FILL x002D             ; x3231 x002D
        .FILL x002D             ; x3232 x002D
        .FILL x002D             ; x3233 x002D
        .FILL x002D             ; x3234 x002D
        .FILL x002D             ; x3235 x002D
        .FILL x002D             ; x3236 x002D
        .FILL x002D             ; x3237 x002D
        .FILL x002D             ; x3238 x002D
        .FILL x002D             ; x3239 x002D
        .FILL x002D             ; x323A x002D
        .FILL x002D             ; x323B x002D
        .FILL x002D             ; x323C x002D
        .FILL x002D             ; x323D x002D
        .FILL x002D             ; x323E x002D
        .FILL x002D             ; x323F x002D
        .FILL x002B             ; x3240 x002B
        .FILL x0000             ; x3241 x0000
L_3242  .FILL x007C             ; x3242 x007C
        .FILL x0020             ; x3243 x0020
        .FILL x0020             ; x3244 x0020
        .FILL x0020             ; x3245 x0020
        .FILL x0020             ; x3246 x0020
        .FILL x0020             ; x3247 x0020
        .FILL x0020             ; x3248 x0020
        .FILL x0020             ; x3249 x0020
        .FILL x0020             ; x324A x0020
        .FILL x0020             ; x324B x0020
        .FILL x0020             ; x324C x0020
        .FILL x0020             ; x324D x0020
        .FILL x0020             ; x324E x0020
        .FILL x0020             ; x324F x0020
        .FILL x0020             ; x3250 x0020
        .FILL x0020             ; x3251 x0020
        .FILL x0020             ; x3252 x0020
        .FILL x0020             ; x3253 x0020
        .FILL x0020             ; x3254 x0020
        .FILL x0020             ; x3255 x0020
        .FILL x0020             ; x3256 x0020
        .FILL x0020             ; x3257 x0020
        .FILL x0020             ; x3258 x0020
        .FILL x0020             ; x3259 x0020
        .FILL x0020             ; x325A x0020
        .FILL x0020             ; x325B x0020
        .FILL x0020             ; x325C x0020
        .FILL x007C             ; x325D x007C
        .FILL x0000             ; x325E x0000
L_325F  .FILL x007C             ; x325F x007C
        .FILL x0020             ; x3260 x0020
        .FILL x0000             ; x3261 x0000
L_3262  .FILL x0020             ; x3262 x0020
        .FILL x007C             ; x3263 x007C
        .FILL x000A             ; x3264 x000A
        .FILL x0000             ; x3265 x0000
L_3266  .FILL x0020             ; x3266 x0020
L_3267  .FILL x000A             ; x3267 x000A
L_3268  .FILL x0020             ; x3268 x0020
        .FILL x0020             ; x3269 x0020
        .FILL x0020             ; x326A x0020
        .FILL x0020             ; x326B x0020
        .FILL x0000             ; x326C x0000
        .FILL x0020             ; x326D x0020
        .FILL x0032             ; x326E x0032
        .FILL x0020             ; x326F x0020
        .FILL x0020             ; x3270 x0020
        .FILL x0000             ; x3271 x0000
        .FILL x0020             ; x3272 x0020
        .FILL x0034             ; x3273 x0034
        .FILL x0020             ; x3274 x0020
        .FILL x0020             ; x3275 x0020
        .FILL x0000             ; x3276 x0000
        .FILL x0020             ; x3277 x0020
        .FILL x0038             ; x3278 x0038
        .FILL x0020             ; x3279 x0020
        .FILL x0020             ; x327A x0020
        .FILL x0000             ; x327B x0000
        .FILL x0020             ; x327C x0020
        .FILL x0031             ; x327D x0031
        .FILL x0036             ; x327E x0036
        .FILL x0020             ; x327F x0020
        .FILL x0000             ; x3280 x0000
        .FILL x0020             ; x3281 x0020
        .FILL x0033             ; x3282 x0033
        .FILL x0032             ; x3283 x0032
        .FILL x0020             ; x3284 x0020
        .FILL x0000             ; x3285 x0000
        .FILL x0020             ; x3286 x0020
        .FILL x0036             ; x3287 x0036
        .FILL x0034             ; x3288 x0034
        .FILL x0020             ; x3289 x0020
        .FILL x0000             ; x328A x0000
        .FILL x0031             ; x328B x0031
        .FILL x0032             ; x328C x0032
        .FILL x0038             ; x328D x0038
        .FILL x0020             ; x328E x0020
        .FILL x0000             ; x328F x0000
        .FILL x0032             ; x3290 x0032
        .FILL x0035             ; x3291 x0035
        .FILL x0036             ; x3292 x0036
        .FILL x0020             ; x3293 x0020
        .FILL x0000             ; x3294 x0000
        .FILL x0035             ; x3295 x0035
        .FILL x0031             ; x3296 x0031
        .FILL x0032             ; x3297 x0032
        .FILL x0020             ; x3298 x0020
        .FILL x0000             ; x3299 x0000
        .FILL x0031             ; x329A x0031
        .FILL x0030             ; x329B x0030
        .FILL x0032             ; x329C x0032
        .FILL x0034             ; x329D x0034
        .FILL x0000             ; x329E x0000
        .FILL x0032             ; x329F x0032
        .FILL x0030             ; x32A0 x0030
        .FILL x0034             ; x32A1 x0034
        .FILL x0038             ; x32A2 x0038
        .FILL x0000             ; x32A3 x0000
        .FILL x0034             ; x32A4 x0034
        .FILL x0030             ; x32A5 x0030
        .FILL x0039             ; x32A6 x0039
        .FILL x0036             ; x32A7 x0036
        .FILL x0000             ; x32A8 x0000
        .FILL x0038             ; x32A9 x0038
        .FILL x0031             ; x32AA x0031
        .FILL x0039             ; x32AB x0039
        .FILL x0032             ; x32AC x0032
        .FILL x0000             ; x32AD x0000
        .FILL x0032             ; x32AE x0032
        .FILL x005E             ; x32AF x005E
        .FILL x0031             ; x32B0 x0031
        .FILL x0034             ; x32B1 x0034
        .FILL x0000             ; x32B2 x0000
        .FILL x0032             ; x32B3 x0032
        .FILL x005E             ; x32B4 x005E
        .FILL x0031             ; x32B5 x0031
        .FILL x0035             ; x32B6 x0035
        .FILL x0000             ; x32B7 x0000
        .FILL x0032             ; x32B8 x0032
        .FILL x005E             ; x32B9 x005E
        .FILL x0031             ; x32BA x0031
        .FILL x0036             ; x32BB x0036
        .FILL x0000             ; x32BC x0000
L_32BD  STR R0, R6, #-1         ; x32BD x71BF
        STR R1, R6, #-2         ; x32BE x73BE
        STR R2, R6, #-3         ; x32BF x75BD
        STR R7, R6, #-4         ; x32C0 x7FBC
        ADD R6, R6, #-4         ; x32C1 x1DBC
        LD R0, L_32D0           ; x32C2 x200D
        LD R1, L_32D3           ; x32C3 x220F
        JSR L_3320              ; x32C4 x485B
        LD R1, L_32D1           ; x32C5 x220B
        JSR L_3334              ; x32C6 x486D
        ST R0, L_32D0           ; x32C7 x3008
        LDR R1, R6, #3          ; x32C8 x6383
        JSR L_3320              ; x32C9 x4856
        LDR R7, R6, #0          ; x32CA x6F80
        LDR R2, R6, #1          ; x32CB x6581
        LDR R1, R6, #2          ; x32CC x6382
        ADD R6, R6, #4          ; x32CD x1DA4
        RET                     ; x32CE xC1C0
L_32CF  .FILL x0000             ; x32CF x0000
L_32D0  .FILL xC20D             ; x32D0 xC20D
L_32D1  .FILL x0007             ; x32D1 x0007
        STR R7, R7, #-1         ; x32D2 x7FFF
L_32D3  .FILL x1249             ; x32D3 x1249
L_32D4  STR R0, R6, #-1         ; x32D4 x71BF
        STR R1, R6, #-2         ; x32D5 x73BE
        STR R7, R6, #-3         ; x32D6 x7FBD
        ADD R6, R6, #-3         ; x32D7 x1DBD
L_32D8  LDR R0, R6, #2          ; x32D8 x6182
        PUTS                    ; x32D9 xF022
        JSR L_330F              ; x32DA x4834
        OUT                     ; x32DB xF021
        ADD R1, R0, #0          ; x32DC x1220
        LD R0, L_330E           ; x32DD x2030
        OUT                     ; x32DE xF021
        LD R0, L_330C           ; x32DF x202C
        ADD R0, R0, R1          ; x32E0 x1001
        BRz L_32EC              ; x32E1 x040A
        LD R0, L_330D           ; x32E2 x202A
        ADD R0, R0, R1          ; x32E3 x1001
        BRz L_32EA              ; x32E4 x0405
        ADD R0, R1, #0          ; x32E5 x1060
        OUT                     ; x32E6 xF021
        LEA R0, L_32F3          ; x32E7 xE00B
        PUTS                    ; x32E8 xF022
        BR L_32D8               ; x32E9 x0FEE
L_32EA  AND R0, R0, #0          ; x32EA x5020
        BR L_32EE               ; x32EB x0E02
L_32EC  AND R0, R0, #0          ; x32EC x5020
        ADD R0, R0, #1          ; x32ED x1021
L_32EE  LDR R7, R6, #0          ; x32EE x6F80
        LDR R1, R6, #1          ; x32EF x6381
        ADD R6, R6, #3          ; x32F0 x1DA3
        ADD R0, R0, #0          ; x32F1 x1020
        RET                     ; x32F2 xC1C0
L_32F3  .FILL x0020             ; x32F3 x0020
        .FILL x0069             ; x32F4 x0069
        .FILL x0073             ; x32F5 x0073
        .FILL x0020             ; x32F6 x0020
        .FILL x006E             ; x32F7 x006E
        .FILL x006F             ; x32F8 x006F
        .FILL x0074             ; x32F9 x0074
        .FILL x0020             ; x32FA x0020
        .FILL x0061             ; x32FB x0061
        .FILL x0020             ; x32FC x0020
        .FILL x0076             ; x32FD x0076
        .FILL x0061             ; x32FE x0061
        .FILL x006C             ; x32FF x006C
        .FILL x0069             ; x3300 x0069
        .FILL x0064             ; x3301 x0064
        .FILL x0020             ; x3302 x0020
        .FILL x0069             ; x3303 x0069
        .FILL x006E             ; x3304 x006E
        .FILL x0070             ; x3305 x0070
        .FILL x0075             ; x3306 x0075
        .FILL x0074             ; x3307 x0074
        .FILL x002E             ; x3308 x002E
        .FILL x000A             ; x3309 x000A
        .FILL x000A             ; x330A x000A
        .FILL x0000             ; x330B x0000
L_330C  .FILL xFF87             ; x330C xFF87
L_330D  .FILL xFF92             ; x330D xFF92
L_330E  .FILL x000A             ; x330E x000A
L_330F  STR R1, R6, #-1         ; x330F x73BF
        ADD R6, R6, #-1         ; x3310 x1DBF
        AND R1, R1, #0          ; x3311 x5260
L_3312  ADD R1, R1, #1          ; x3312 x1261
        LDI R0, L_331D          ; x3313 xA009
        BRzp L_3312             ; x3314 x07FD
        LD R0, L_331F           ; x3315 x2009
        AND R1, R1, R0          ; x3316 x5240
        LDI R0, L_331E          ; x3317 xA006
        ST R1, L_32D0           ; x3318 x33B7
        ST R1, L_32CF           ; x3319 x33B5
        LDR R1, R6, #0          ; x331A x6380
        ADD R6, R6, #1          ; x331B x1DA1
        RET                     ; x331C xC1C0
L_331D  .FILL xFE00             ; x331D xFE00
L_331E  .FILL xFE02             ; x331E xFE02
L_331F  STR R7, R7, #-1         ; x331F x7FFF
L_3320  STR R1, R6, #-1         ; x3320 x73BF
        STR R2, R6, #-2         ; x3321 x75BE
        STR R3, R6, #-3         ; x3322 x77BD
        ADD R6, R6, #-3         ; x3323 x1DBD
        NOT R2, R1              ; x3324 x947F
        ADD R2, R2, #1          ; x3325 x14A1
        BRz L_3333              ; x3326 x040C
        AND R1, R1, #0          ; x3327 x5260
L_3328  ADD R1, R1, #1          ; x3328 x1261
        ADD R0, R0, R2          ; x3329 x1002
        BRp L_3328              ; x332A x03FD
        BRz L_332F              ; x332B x0403
        LDR R2, R6, #2          ; x332C x6582
        ADD R1, R1, #-1         ; x332D x127F
        ADD R0, R0, R2          ; x332E x1002
L_332F  LDR R3, R6, #0          ; x332F x6780
        LDR R2, R6, #1          ; x3330 x6581
        ADD R6, R6, #3          ; x3331 x1DA3
        RET                     ; x3332 xC1C0
L_3333  HALT                    ; x3333 xF025
L_3334  ADD R0, R0, #0          ; x3334 x1020
        BRz L_334C              ; x3335 x0416
        ADD R1, R1, #0          ; x3336 x1260
        BRz L_334C              ; x3337 x0414
        STR R1, R6, #-1         ; x3338 x73BF
        STR R2, R6, #-2         ; x3339 x75BE
        STR R3, R6, #-3         ; x333A x77BD
        STR R4, R6, #-4         ; x333B x79BC
        ADD R6, R6, #-4         ; x333C x1DBC
        AND R2, R2, #0          ; x333D x54A0
        ADD R3, R2, #1          ; x333E x16A1
L_333F  AND R4, R0, R3          ; x333F x5803
        BRnz L_3342             ; x3340 x0C01
        ADD R2, R2, R1          ; x3341 x1481
L_3342  ADD R1, R1, R1          ; x3342 x1241
        ADD R3, R3, R3          ; x3343 x16C3
        BRp L_333F              ; x3344 x03FA
        ADD R0, R2, #0          ; x3345 x10A0
        LDR R4, R6, #0          ; x3346 x6980
        LDR R3, R6, #1          ; x3347 x6781
        LDR R2, R6, #2          ; x3348 x6582
        LDR R1, R6, #3          ; x3349 x6383
        ADD R6, R6, #4          ; x334A x1DA4
        RET                     ; x334B xC1C0
L_334C  AND R0, R0, #0          ; x334C x5020
        RET                     ; x334D xC1C0
.END
