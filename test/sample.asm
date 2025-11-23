        ORG 0000H

; --- Register-only tests ---

START:  MVI A, 05H        ; A = 0x05
        MVI B, 03H        ; B = 0x03
        MVI C, 0FFH       ; C = 0xFF

        INR B             ; B = 0x04
        DCR B             ; B = 0x03 (back to original)

        INR C             ; C: 0xFF -> 0x00 (Z flag should be set)
        DCR C             ; C: 0x00 -> 0xFF

; --- MOV tests ---

        MOV D, A          ; D = A = 0x05
        MOV E, B          ; E = B = 0x03
        MOV A, E          ; A = E = 0x03

; --- Memory (M) tests via HL ---

        LXI  H, 4000H     ; HL = 0x4000
        MVI  M, 10H       ; [4000] = 0x10
        INR  M            ; [4000] = 0x11
        DCR  M            ; [4000] = 0x10 (back to original)

; Test INR/DCR on H and L too
        INR  L            ; L: 0x00 -> 0x01 (HL = 0x4001)
        INR  H            ; H: 0x40 -> 0x41 (HL = 0x4101)

        HLT               ; stop here

