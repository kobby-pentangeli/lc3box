.ORIG x3000
        LEA R0, L_3004          ; x3000 xE003
        PUTS                    ; x3001 xF022
        GETC                    ; x3002 xF020
        BR L_303D               ; x3003 x0E39
L_3004  .FILL x0057             ; x3004 x0057
        .FILL x0065             ; x3005 x0065
        .FILL x006C             ; x3006 x006C
        .FILL x0063             ; x3007 x0063
        .FILL x006F             ; x3008 x006F
        .FILL x006D             ; x3009 x006D
        .FILL x0065             ; x300A x0065
        .FILL x0020             ; x300B x0020
        .FILL x0074             ; x300C x0074
        .FILL x006F             ; x300D x006F
        .FILL x0020             ; x300E x0020
        .FILL x004C             ; x300F x004C
        .FILL x0043             ; x3010 x0043
        .FILL x0033             ; x3011 x0033
        .FILL x0020             ; x3012 x0020
        .FILL x0052             ; x3013 x0052
        .FILL x006F             ; x3014 x006F
        .FILL x0067             ; x3015 x0067
        .FILL x0075             ; x3016 x0075
        .FILL x0065             ; x3017 x0065
        .FILL x002E             ; x3018 x002E
        .FILL x000A             ; x3019 x000A
        .FILL x0055             ; x301A x0055
        .FILL x0073             ; x301B x0073
        .FILL x0065             ; x301C x0065
        .FILL x0020             ; x301D x0020
        .FILL x0057             ; x301E x0057
        .FILL x0053             ; x301F x0053
        .FILL x0041             ; x3020 x0041
        .FILL x0044             ; x3021 x0044
        .FILL x0020             ; x3022 x0020
        .FILL x0074             ; x3023 x0074
        .FILL x006F             ; x3024 x006F
        .FILL x0020             ; x3025 x0020
        .FILL x006D             ; x3026 x006D
        .FILL x006F             ; x3027 x006F
        .FILL x0076             ; x3028 x0076
        .FILL x0065             ; x3029 x0065
        .FILL x002E             ; x302A x002E
        .FILL x000A             ; x302B x000A
        .FILL x0050             ; x302C x0050
        .FILL x0072             ; x302D x0072
        .FILL x0065             ; x302E x0065
        .FILL x0073             ; x302F x0073
        .FILL x0073             ; x3030 x0073
        .FILL x0020             ; x3031 x0020
        .FILL x0061             ; x3032 x0061
        .FILL x006E             ; x3033 x006E
        .FILL x0079             ; x3034 x0079
        .FILL x0020             ; x3035 x0020
        .FILL x006B             ; x3036 x006B
        .FILL x0065             ; x3037 x0065
        .FILL x0079             ; x3038 x0079
        .FILL x002E             ; x3039 x002E
        .FILL x002E             ; x303A x002E
        .FILL x000A             ; x303B x000A
        .FILL x0000             ; x303C x0000
L_303D  LD R6, L_3091           ; x303D x2C53
        JSR L_3114              ; x303E x48D5
L_303F  LD R0, L_3097           ; x303F x2057
        BRp L_3044              ; x3040 x0203
        JSR L_30D5              ; x3041 x4893
        JSR L_3099              ; x3042 x4856
        HALT                    ; x3043 xF025
L_3044  AND R0, R0, #0          ; x3044 x5020
        ST R0, L_3097           ; x3045 x3051
        LEA R0, L_304D          ; x3046 xE006
        PUTS                    ; x3047 xF022
        GETC                    ; x3048 xF020
        LD R1, L_3090           ; x3049 x2246
        ADD R1, R0, R1          ; x304A x1201
        BRnp L_303D             ; x304B x0BF1
        HALT                    ; x304C xF025
L_304D  .FILL x0059             ; x304D x0059
        .FILL x006F             ; x304E x006F
        .FILL x0075             ; x304F x0075
        .FILL x0020             ; x3050 x0020
        .FILL x0073             ; x3051 x0073
        .FILL x0075             ; x3052 x0075
        .FILL x0072             ; x3053 x0072
        .FILL x0076             ; x3054 x0076
        .FILL x0069             ; x3055 x0069
        .FILL x0076             ; x3056 x0076
        .FILL x0065             ; x3057 x0065
        .FILL x0064             ; x3058 x0064
        .FILL x0021             ; x3059 x0021
        .FILL x000A             ; x305A x000A
        .FILL x004F             ; x305B x004F
        .FILL x006E             ; x305C x006E
        .FILL x0020             ; x305D x0020
        .FILL x0074             ; x305E x0074
        .FILL x006F             ; x305F x006F
        .FILL x0020             ; x3060 x0020
        .FILL x0061             ; x3061 x0061
        .FILL x006E             ; x3062 x006E
        .FILL x006F             ; x3063 x006F
        .FILL x0074             ; x3064 x0074
        .FILL x0068             ; x3065 x0068
        .FILL x0065             ; x3066 x0065
        .FILL x0072             ; x3067 x0072
        .FILL x0020             ; x3068 x0020
        .FILL x0064             ; x3069 x0064
        .FILL x0075             ; x306A x0075
        .FILL x006E             ; x306B x006E
        .FILL x0067             ; x306C x0067
        .FILL x0065             ; x306D x0065
        .FILL x006F             ; x306E x006F
        .FILL x006E             ; x306F x006E
        .FILL x003F             ; x3070 x003F
        .FILL x0020             ; x3071 x0020
        .FILL x0028             ; x3072 x0028
        .FILL x006E             ; x3073 x006E
        .FILL x0029             ; x3074 x0029
        .FILL x006F             ; x3075 x006F
        .FILL x0020             ; x3076 x0020
        .FILL x006F             ; x3077 x006F
        .FILL x0072             ; x3078 x0072
        .FILL x0020             ; x3079 x0020
        .FILL x0061             ; x307A x0061
        .FILL x006E             ; x307B x006E
        .FILL x0079             ; x307C x0079
        .FILL x0020             ; x307D x0020
        .FILL x006B             ; x307E x006B
        .FILL x0065             ; x307F x0065
        .FILL x0079             ; x3080 x0079
        .FILL x0020             ; x3081 x0020
        .FILL x0074             ; x3082 x0074
        .FILL x006F             ; x3083 x006F
        .FILL x0020             ; x3084 x0020
        .FILL x0063             ; x3085 x0063
        .FILL x006F             ; x3086 x006F
        .FILL x006E             ; x3087 x006E
        .FILL x0074             ; x3088 x0074
        .FILL x0069             ; x3089 x0069
        .FILL x006E             ; x308A x006E
        .FILL x0075             ; x308B x0075
        .FILL x0065             ; x308C x0065
        .FILL x002E             ; x308D x002E
        .FILL x000A             ; x308E x000A
        .FILL x0000             ; x308F x0000
L_3090  .FILL xFF92             ; x3090 xFF92
L_3091  JSRR R0                 ; x3091 x4000
        .FILL x0041             ; x3092 x0041
L_3093  .FILL x0020             ; x3093 x0020
L_3094  .FILL x0010             ; x3094 x0010
L_3095  .FILL x0000             ; x3095 x0000
L_3096  .FILL x0000             ; x3096 x0000
L_3097  .FILL x0000             ; x3097 x0000
L_3098  ST R2, #-256            ; x3098 x3500
L_3099  LD R1, L_3095           ; x3099 x23FB
        LD R2, L_3096           ; x309A x25FB
        GETC                    ; x309B xF020
        LD R3, L_30D0           ; x309C x2633
        ADD R3, R0, R3          ; x309D x1603
        BRz L_30A9              ; x309E x040A
        LD R3, L_30D2           ; x309F x2632
        ADD R3, R0, R3          ; x30A0 x1603
        BRz L_30AB              ; x30A1 x0409
        LD R3, L_30D1           ; x30A2 x262E
        ADD R3, R0, R3          ; x30A3 x1603
        BRz L_30AD              ; x30A4 x0408
        LD R3, L_30D3           ; x30A5 x262D
        ADD R3, R0, R3          ; x30A6 x1603
        BRz L_30AF              ; x30A7 x0407
        BR L_3099               ; x30A8 x0FF0
L_30A9  ADD R2, R2, #-1         ; x30A9 x14BF
        BR L_30B1               ; x30AA x0E06
L_30AB  ADD R2, R2, #1          ; x30AB x14A1
        BR L_30B1               ; x30AC x0E04
L_30AD  ADD R1, R1, #-1         ; x30AD x127F
        BR L_30B1               ; x30AE x0E02
L_30AF  ADD R1, R1, #1          ; x30AF x1261
        BR L_30B1               ; x30B0 x0E00
L_30B1  AND R1, R1, #-1         ; x30B1 x527F
        AND R2, R2, #15         ; x30B2 x54AF
        STR R1, R6, #-1         ; x30B3 x73BF
        STR R2, R6, #-2         ; x30B4 x75BE
        ADD R6, R6, #-2         ; x30B5 x1DBE
        JSR L_3101              ; x30B6 x484A
        LDR R3, R3, #0          ; x30B7 x66C0
        BRz L_30BD              ; x30B8 x0404
        LD R4, L_30D4           ; x30B9 x281A
        ADD R3, R3, R4          ; x30BA x16C4
        BRz L_30CC              ; x30BB x0410
        JSR L_303F              ; x30BC x4F82
L_30BD  LD R1, L_3095           ; x30BD x23D7
        LD R2, L_3096           ; x30BE x25D7
        JSR L_3101              ; x30BF x4841
        AND R4, R4, #0          ; x30C0 x5920
        STR R4, R3, #0          ; x30C1 x78C0
        LDR R2, R6, #0          ; x30C2 x6580
        LDR R1, R6, #1          ; x30C3 x6381
        ADD R6, R6, #2          ; x30C4 x1DA2
        JSR L_3101              ; x30C5 x483B
        AND R4, R4, #0          ; x30C6 x5920
        ADD R4, R4, #2          ; x30C7 x1922
        STR R4, R3, #0          ; x30C8 x78C0
        ST R1, L_3095           ; x30C9 x33CB
        ST R2, L_3096           ; x30CA x35CB
        JSR L_303F              ; x30CB x4F73
L_30CC  AND R4, R4, #0          ; x30CC x5920
        ADD R4, R4, #1          ; x30CD x1921
        ST R4, L_3097           ; x30CE x39C8
        JSR L_303F              ; x30CF x4F6F
L_30D0  .FILL xFF89             ; x30D0 xFF89
L_30D1  .FILL xFF9F             ; x30D1 xFF9F
L_30D2  .FILL xFF8D             ; x30D2 xFF8D
L_30D3  .FILL xFF9C             ; x30D3 xFF9C
L_30D4  .FILL xFFFC             ; x30D4 xFFFC
L_30D5  STR R7, R6, #-1         ; x30D5 x7FBF
        ADD R6, R6, #-1         ; x30D6 x1DBF
        LD R3, L_3093           ; x30D7 x27BB
        LD R4, L_3094           ; x30D8 x29BB
        LD R5, L_3098           ; x30D9 x2BBE
        LEA R0, L_30F5          ; x30DA xE01A
        PUTS                    ; x30DB xF022
L_30DC  LDR R1, R5, #0          ; x30DC x6340
        LEA R2, L_30EF          ; x30DD xE411
        ADD R2, R2, R1          ; x30DE x1481
        LDR R0, R2, #0          ; x30DF x6080
        OUT                     ; x30E0 xF021
        ADD R5, R5, #1          ; x30E1 x1B61
        ADD R3, R3, #-1         ; x30E2 x16FF
        BRp L_30DC              ; x30E3 x03F8
        LD R0, L_30EE           ; x30E4 x2009
        OUT                     ; x30E5 xF021
        LD R3, L_3093           ; x30E6 x27AC
        ADD R4, R4, #-1         ; x30E7 x193F
        BRp L_30DC              ; x30E8 x03F3
        LD R0, L_30EE           ; x30E9 x2004
        OUT                     ; x30EA xF021
        LDR R7, R6, #0          ; x30EB x6F80
        ADD R6, R6, #1          ; x30EC x1DA1
        RET                     ; x30ED xC1C0
L_30EE  .FILL x000A             ; x30EE x000A
L_30EF  .FILL x0020             ; x30EF x0020
        .FILL x0023             ; x30F0 x0023
        .FILL x0040             ; x30F1 x0040
L_30F2  .FILL x004B             ; x30F2 x004B
        .FILL x0044             ; x30F3 x0044
        .FILL x0000             ; x30F4 x0000
L_30F5  .FILL x001B             ; x30F5 x001B
        .FILL x005B             ; x30F6 x005B
        .FILL x0032             ; x30F7 x0032
        .FILL x004A             ; x30F8 x004A
        .FILL x001B             ; x30F9 x001B
        .FILL x005B             ; x30FA x005B
        .FILL x0048             ; x30FB x0048
        .FILL x001B             ; x30FC x001B
        .FILL x005B             ; x30FD x005B
        .FILL x0033             ; x30FE x0033
        .FILL x004A             ; x30FF x004A
        .FILL x0000             ; x3100 x0000
L_3101  STR R0, R6, #-1         ; x3101 x71BF
        STR R4, R6, #-2         ; x3102 x79BE
        STR R5, R6, #-3         ; x3103 x7BBD
        STR R7, R6, #-4         ; x3104 x7FBC
        ADD R6, R6, #-4         ; x3105 x1DBC
        LD R3, L_3098           ; x3106 x2791
        ADD R3, R3, R1          ; x3107 x16C1
        ADD R4, R2, #0          ; x3108 x18A0
        BRz L_310E              ; x3109 x0404
        LD R5, L_3093           ; x310A x2B88
L_310B  ADD R3, R3, R5          ; x310B x16C5
        ADD R4, R4, #-1         ; x310C x193F
        BRp L_310B              ; x310D x03FD
L_310E  LDR R7, R6, #0          ; x310E x6F80
        LDR R5, R6, #1          ; x310F x6B81
        LDR R4, R6, #2          ; x3110 x6982
        LDR R0, R6, #3          ; x3111 x6183
        ADD R6, R6, #4          ; x3112 x1DA4
        RET                     ; x3113 xC1C0
L_3114  STR R7, R6, #-1         ; x3114 x7FBF
        ADD R6, R6, #-1         ; x3115 x1DBF
        LD R1, L_3093           ; x3116 x237C
        LD R2, L_3094           ; x3117 x257C
        AND R3, R3, #0          ; x3118 x56E0
        ADD R3, R3, #1          ; x3119 x16E1
        LD R5, L_3098           ; x311A x2B7D
L_311B  STR R3, R5, #0          ; x311B x7740
        ADD R5, R5, #1          ; x311C x1B61
        ADD R1, R1, #-1         ; x311D x127F
        BRp L_311B              ; x311E x03FC
        LD R1, L_3093           ; x311F x2373
        ADD R2, R2, #-1         ; x3120 x14BF
        BRp L_311B              ; x3121 x03F9
        JSR L_3149              ; x3122 x4826
        LD R1, L_3094           ; x3123 x2370
        JSR L_3167              ; x3124 x4842
        ADD R2, R0, #0          ; x3125 x1420
        AND R1, R1, #0          ; x3126 x5260
        JSR L_3101              ; x3127 x4FD9
        AND R4, R4, #0          ; x3128 x5920
        ADD R4, R4, #2          ; x3129 x1922
        STR R4, R3, #0          ; x312A x78C0
        ST R1, L_3095           ; x312B x3369
        ST R2, L_3096           ; x312C x3569
        LD R4, L_3093           ; x312D x2965
        ADD R4, R4, #-1         ; x312E x193F
        NOT R4, R4              ; x312F x993F
        ADD R4, R4, #1          ; x3130 x1921
L_3131  ADD R1, R1, #1          ; x3131 x1261
        JSR L_3101              ; x3132 x4FCE
        AND R5, R5, #0          ; x3133 x5B60
        STR R5, R3, #0          ; x3134 x7AC0
        STR R1, R6, #-1         ; x3135 x73BF
        ADD R6, R6, #-1         ; x3136 x1DBF
        JSR L_3149              ; x3137 x4811
        AND R1, R1, #0          ; x3138 x5260
        ADD R1, R1, #3          ; x3139 x1263
        JSR L_3167              ; x313A x482C
        ADD R0, R0, #-1         ; x313B x103F
        ADD R2, R2, R0          ; x313C x1480
        AND R2, R2, #15         ; x313D x54AF
        LDR R1, R6, #0          ; x313E x6380
        ADD R6, R6, #1          ; x313F x1DA1
        JSR L_3101              ; x3140 x4FC0
        STR R5, R3, #0          ; x3141 x7AC0
        ADD R5, R1, R4          ; x3142 x1A44
        BRn L_3131              ; x3143 x09ED
        ADD R5, R5, #4          ; x3144 x1B64
        STR R5, R3, #0          ; x3145 x7AC0
        LDR R7, R6, #0          ; x3146 x6F80
        ADD R6, R6, #1          ; x3147 x1DA1
        RET                     ; x3148 xC1C0
L_3149  STR R1, R6, #-1         ; x3149 x73BF
        STR R2, R6, #-2         ; x314A x75BE
        STR R3, R6, #-3         ; x314B x77BD
        STR R4, R6, #-4         ; x314C x79BC
        STR R5, R6, #-5         ; x314D x7BBB
        STR R7, R6, #-6         ; x314E x7FBA
        ADD R6, R6, #-6         ; x314F x1DBA
        LD R1, L_3164           ; x3150 x2213
        LD R2, L_3163           ; x3151 x2411
        AND R0, R0, #0          ; x3152 x5020
L_3153  ADD R0, R0, R2          ; x3153 x1002
        ADD R1, R1, #-1         ; x3154 x127F
        BRp L_3153              ; x3155 x03FD
        LD R1, L_3165           ; x3156 x220E
        ADD R0, R0, R1          ; x3157 x1001
        LD R1, L_3166           ; x3158 x220D
        AND R0, R0, R1          ; x3159 x5001
        ST R0, L_3163           ; x315A x3008
        LDR R7, R6, #0          ; x315B x6F80
        LDR R5, R6, #1          ; x315C x6B81
        LDR R4, R6, #2          ; x315D x6982
        LDR R3, R6, #3          ; x315E x6783
        LDR R2, R6, #4          ; x315F x6584
        LDR R1, R6, #5          ; x3160 x6385
        ADD R6, R6, #6          ; x3161 x1DA6
        RET                     ; x3162 xC1C0
L_3163  LDI R6, #52             ; x3163 xAC34
L_3164  ST R5, L_30F2           ; x3164 x3B8D
L_3165  .FILL x0083             ; x3165 x0083
L_3166  STR R7, R7, #-1         ; x3166 x7FFF
L_3167  STR R2, R6, #-1         ; x3167 x75BF
        STR R3, R6, #-2         ; x3168 x77BE
        STR R4, R6, #-3         ; x3169 x79BD
        STR R5, R6, #-4         ; x316A x7BBC
        STR R7, R6, #-5         ; x316B x7FBB
        ADD R6, R6, #-5         ; x316C x1DBB
        NOT R1, R1              ; x316D x927F
        ADD R1, R1, #1          ; x316E x1261
        BRz L_3175              ; x316F x0405
        ADD R2, R0, R1          ; x3170 x1401
        BRn L_3175              ; x3171 x0803
L_3172  ADD R0, R0, R1          ; x3172 x1001
        ADD R2, R0, R1          ; x3173 x1401
        BRzp L_3172             ; x3174 x07FD
L_3175  LDR R7, R6, #0          ; x3175 x6F80
        LDR R5, R6, #1          ; x3176 x6B81
        LDR R4, R6, #2          ; x3177 x6982
        LDR R3, R6, #3          ; x3178 x6783
        LDR R2, R6, #4          ; x3179 x6584
        ADD R6, R6, #5          ; x317A x1DA5
        RET                     ; x317B xC1C0
.END
