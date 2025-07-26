(module
    (func (export "many-params")
        (param
            ;; Define 150 i32 params
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
        )
    )

    (func (export "many-results")
        (result
            ;; Define 150 i32 results
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
        )
        (call $return10 (i32.const 0))
        (call $return10 (i32.const 10))
        (call $return10 (i32.const 20))
        (call $return10 (i32.const 30))
        (call $return10 (i32.const 40))
        (call $return10 (i32.const 50))
        (call $return10 (i32.const 60))
        (call $return10 (i32.const 70))
        (call $return10 (i32.const 80))
        (call $return10 (i32.const 90))
        (call $return10 (i32.const 100))
        (call $return10 (i32.const 110))
        (call $return10 (i32.const 120))
        (call $return10 (i32.const 130))
        (call $return10 (i32.const 140))
    )

    (func $return10
        (param i32)
        (result i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
        (i32.add (local.get 0) (i32.const 0))
        (i32.add (local.get 0) (i32.const 1))
        (i32.add (local.get 0) (i32.const 2))
        (i32.add (local.get 0) (i32.const 3))
        (i32.add (local.get 0) (i32.const 4))
        (i32.add (local.get 0) (i32.const 5))
        (i32.add (local.get 0) (i32.const 6))
        (i32.add (local.get 0) (i32.const 7))
        (i32.add (local.get 0) (i32.const 8))
        (i32.add (local.get 0) (i32.const 9))
    )

    (func (export "many-inout")
        (param
            ;; Define 150 i32 params
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
        )
        (result
            ;; Define 150 i32 results
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
        )
        (local.get 0)
        (local.get 1)
        (local.get 2)
        (local.get 3)
        (local.get 4)
        (local.get 5)
        (local.get 6)
        (local.get 7)
        (local.get 8)
        (local.get 9)
        (local.get 10)
        (local.get 11)
        (local.get 12)
        (local.get 13)
        (local.get 14)
        (local.get 15)
        (local.get 16)
        (local.get 17)
        (local.get 18)
        (local.get 19)
        (local.get 20)
        (local.get 21)
        (local.get 22)
        (local.get 23)
        (local.get 24)
        (local.get 25)
        (local.get 26)
        (local.get 27)
        (local.get 28)
        (local.get 29)
        (local.get 30)
        (local.get 31)
        (local.get 32)
        (local.get 33)
        (local.get 34)
        (local.get 35)
        (local.get 36)
        (local.get 37)
        (local.get 38)
        (local.get 39)
        (local.get 40)
        (local.get 41)
        (local.get 42)
        (local.get 43)
        (local.get 44)
        (local.get 45)
        (local.get 46)
        (local.get 47)
        (local.get 48)
        (local.get 49)
        (local.get 50)
        (local.get 51)
        (local.get 52)
        (local.get 53)
        (local.get 54)
        (local.get 55)
        (local.get 56)
        (local.get 57)
        (local.get 58)
        (local.get 59)
        (local.get 60)
        (local.get 61)
        (local.get 62)
        (local.get 63)
        (local.get 64)
        (local.get 65)
        (local.get 66)
        (local.get 67)
        (local.get 68)
        (local.get 69)
        (local.get 70)
        (local.get 71)
        (local.get 72)
        (local.get 73)
        (local.get 74)
        (local.get 75)
        (local.get 76)
        (local.get 77)
        (local.get 78)
        (local.get 79)
        (local.get 80)
        (local.get 81)
        (local.get 82)
        (local.get 83)
        (local.get 84)
        (local.get 85)
        (local.get 86)
        (local.get 87)
        (local.get 88)
        (local.get 89)
        (local.get 90)
        (local.get 91)
        (local.get 92)
        (local.get 93)
        (local.get 94)
        (local.get 95)
        (local.get 96)
        (local.get 97)
        (local.get 98)
        (local.get 99)
        (local.get 100)
        (local.get 101)
        (local.get 102)
        (local.get 103)
        (local.get 104)
        (local.get 105)
        (local.get 106)
        (local.get 107)
        (local.get 108)
        (local.get 109)
        (local.get 110)
        (local.get 111)
        (local.get 112)
        (local.get 113)
        (local.get 114)
        (local.get 115)
        (local.get 116)
        (local.get 117)
        (local.get 118)
        (local.get 119)
        (local.get 120)
        (local.get 121)
        (local.get 122)
        (local.get 123)
        (local.get 124)
        (local.get 125)
        (local.get 126)
        (local.get 127)
        (local.get 128)
        (local.get 129)
        (local.get 130)
        (local.get 131)
        (local.get 132)
        (local.get 133)
        (local.get 134)
        (local.get 135)
        (local.get 136)
        (local.get 137)
        (local.get 138)
        (local.get 139)
        (local.get 140)
        (local.get 141)
        (local.get 142)
        (local.get 143)
        (local.get 144)
        (local.get 145)
        (local.get 146)
        (local.get 147)
        (local.get 148)
        (local.get 149)
    )

    (func (export "many-inout-reverse")
        (param
            ;; Define 150 i32 params
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
        )
        (result
            ;; Define 150 i32 results
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
            i32 i32 i32 i32 i32 i32 i32 i32 i32 i32
        )
        (local.get 149)
        (local.get 148)
        (local.get 147)
        (local.get 146)
        (local.get 145)
        (local.get 144)
        (local.get 143)
        (local.get 142)
        (local.get 141)
        (local.get 140)
        (local.get 139)
        (local.get 138)
        (local.get 137)
        (local.get 136)
        (local.get 135)
        (local.get 134)
        (local.get 133)
        (local.get 132)
        (local.get 131)
        (local.get 130)
        (local.get 129)
        (local.get 128)
        (local.get 127)
        (local.get 126)
        (local.get 125)
        (local.get 124)
        (local.get 123)
        (local.get 122)
        (local.get 121)
        (local.get 120)
        (local.get 119)
        (local.get 118)
        (local.get 117)
        (local.get 116)
        (local.get 115)
        (local.get 114)
        (local.get 113)
        (local.get 112)
        (local.get 111)
        (local.get 110)
        (local.get 109)
        (local.get 108)
        (local.get 107)
        (local.get 106)
        (local.get 105)
        (local.get 104)
        (local.get 103)
        (local.get 102)
        (local.get 101)
        (local.get 100)
        (local.get 99)
        (local.get 98)
        (local.get 97)
        (local.get 96)
        (local.get 95)
        (local.get 94)
        (local.get 93)
        (local.get 92)
        (local.get 91)
        (local.get 90)
        (local.get 89)
        (local.get 88)
        (local.get 87)
        (local.get 86)
        (local.get 85)
        (local.get 84)
        (local.get 83)
        (local.get 82)
        (local.get 81)
        (local.get 80)
        (local.get 79)
        (local.get 78)
        (local.get 77)
        (local.get 76)
        (local.get 75)
        (local.get 74)
        (local.get 73)
        (local.get 72)
        (local.get 71)
        (local.get 70)
        (local.get 69)
        (local.get 68)
        (local.get 67)
        (local.get 66)
        (local.get 65)
        (local.get 64)
        (local.get 63)
        (local.get 62)
        (local.get 61)
        (local.get 60)
        (local.get 59)
        (local.get 58)
        (local.get 57)
        (local.get 56)
        (local.get 55)
        (local.get 54)
        (local.get 53)
        (local.get 52)
        (local.get 51)
        (local.get 50)
        (local.get 49)
        (local.get 48)
        (local.get 47)
        (local.get 46)
        (local.get 45)
        (local.get 44)
        (local.get 43)
        (local.get 42)
        (local.get 41)
        (local.get 40)
        (local.get 39)
        (local.get 38)
        (local.get 37)
        (local.get 36)
        (local.get 35)
        (local.get 34)
        (local.get 33)
        (local.get 32)
        (local.get 31)
        (local.get 30)
        (local.get 29)
        (local.get 28)
        (local.get 27)
        (local.get 26)
        (local.get 25)
        (local.get 24)
        (local.get 23)
        (local.get 22)
        (local.get 21)
        (local.get 20)
        (local.get 19)
        (local.get 18)
        (local.get 17)
        (local.get 16)
        (local.get 15)
        (local.get 14)
        (local.get 13)
        (local.get 12)
        (local.get 11)
        (local.get 10)
        (local.get 9)
        (local.get 8)
        (local.get 7)
        (local.get 6)
        (local.get 5)
        (local.get 4)
        (local.get 3)
        (local.get 2)
        (local.get 1)
        (local.get 0)
    )
)

(assert_return
    (invoke "many-params"
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) ;; 10
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) ;; 20
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) ;; 30
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) ;; 40
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) ;; 50
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) ;; 60
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) ;; 70
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) ;; 80
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) ;; 90
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) ;; 100
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) ;; 110
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) ;; 120
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) ;; 130
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) ;; 140
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0)
        (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) (i32.const 0) ;; 150
    )
)

(assert_return
    (invoke "many-results")
    (i32.const   0) (i32.const   1) (i32.const   2) (i32.const   3) (i32.const   4)
    (i32.const   5) (i32.const   6) (i32.const   7) (i32.const   8) (i32.const   9) ;; 10
    (i32.const  10) (i32.const  11) (i32.const  12) (i32.const  13) (i32.const  14)
    (i32.const  15) (i32.const  16) (i32.const  17) (i32.const  18) (i32.const  19) ;; 20
    (i32.const  20) (i32.const  21) (i32.const  22) (i32.const  23) (i32.const  24)
    (i32.const  25) (i32.const  26) (i32.const  27) (i32.const  28) (i32.const  29) ;; 30
    (i32.const  30) (i32.const  31) (i32.const  32) (i32.const  33) (i32.const  34)
    (i32.const  35) (i32.const  36) (i32.const  37) (i32.const  38) (i32.const  39) ;; 40
    (i32.const  40) (i32.const  41) (i32.const  42) (i32.const  43) (i32.const  44)
    (i32.const  45) (i32.const  46) (i32.const  47) (i32.const  48) (i32.const  49) ;; 50
    (i32.const  50) (i32.const  51) (i32.const  52) (i32.const  53) (i32.const  54)
    (i32.const  55) (i32.const  56) (i32.const  57) (i32.const  58) (i32.const  59) ;; 60
    (i32.const  60) (i32.const  61) (i32.const  62) (i32.const  63) (i32.const  64)
    (i32.const  65) (i32.const  66) (i32.const  67) (i32.const  68) (i32.const  69) ;; 70
    (i32.const  70) (i32.const  71) (i32.const  72) (i32.const  73) (i32.const  74)
    (i32.const  75) (i32.const  76) (i32.const  77) (i32.const  78) (i32.const  79) ;; 80
    (i32.const  80) (i32.const  81) (i32.const  82) (i32.const  83) (i32.const  84)
    (i32.const  85) (i32.const  86) (i32.const  87) (i32.const  88) (i32.const  89) ;; 90
    (i32.const  90) (i32.const  91) (i32.const  92) (i32.const  93) (i32.const  94)
    (i32.const  95) (i32.const  96) (i32.const  97) (i32.const  98) (i32.const  99) ;; 100
    (i32.const 100) (i32.const 101) (i32.const 102) (i32.const 103) (i32.const 104)
    (i32.const 105) (i32.const 106) (i32.const 107) (i32.const 108) (i32.const 109) ;; 110
    (i32.const 110) (i32.const 111) (i32.const 112) (i32.const 113) (i32.const 114)
    (i32.const 115) (i32.const 116) (i32.const 117) (i32.const 118) (i32.const 119) ;; 120
    (i32.const 120) (i32.const 121) (i32.const 122) (i32.const 123) (i32.const 124)
    (i32.const 125) (i32.const 126) (i32.const 127) (i32.const 128) (i32.const 129) ;; 130
    (i32.const 130) (i32.const 131) (i32.const 132) (i32.const 133) (i32.const 134)
    (i32.const 135) (i32.const 136) (i32.const 137) (i32.const 138) (i32.const 139) ;; 140
    (i32.const 140) (i32.const 141) (i32.const 142) (i32.const 143) (i32.const 144)
    (i32.const 145) (i32.const 146) (i32.const 147) (i32.const 148) (i32.const 149) ;; 150
)

(assert_return
    (invoke "many-inout"
        (i32.const   0) (i32.const   1) (i32.const   2) (i32.const   3) (i32.const   4)
        (i32.const   5) (i32.const   6) (i32.const   7) (i32.const   8) (i32.const   9) ;; 10
        (i32.const  10) (i32.const  11) (i32.const  12) (i32.const  13) (i32.const  14)
        (i32.const  15) (i32.const  16) (i32.const  17) (i32.const  18) (i32.const  19) ;; 20
        (i32.const  20) (i32.const  21) (i32.const  22) (i32.const  23) (i32.const  24)
        (i32.const  25) (i32.const  26) (i32.const  27) (i32.const  28) (i32.const  29) ;; 30
        (i32.const  30) (i32.const  31) (i32.const  32) (i32.const  33) (i32.const  34)
        (i32.const  35) (i32.const  36) (i32.const  37) (i32.const  38) (i32.const  39) ;; 40
        (i32.const  40) (i32.const  41) (i32.const  42) (i32.const  43) (i32.const  44)
        (i32.const  45) (i32.const  46) (i32.const  47) (i32.const  48) (i32.const  49) ;; 50
        (i32.const  50) (i32.const  51) (i32.const  52) (i32.const  53) (i32.const  54)
        (i32.const  55) (i32.const  56) (i32.const  57) (i32.const  58) (i32.const  59) ;; 60
        (i32.const  60) (i32.const  61) (i32.const  62) (i32.const  63) (i32.const  64)
        (i32.const  65) (i32.const  66) (i32.const  67) (i32.const  68) (i32.const  69) ;; 70
        (i32.const  70) (i32.const  71) (i32.const  72) (i32.const  73) (i32.const  74)
        (i32.const  75) (i32.const  76) (i32.const  77) (i32.const  78) (i32.const  79) ;; 80
        (i32.const  80) (i32.const  81) (i32.const  82) (i32.const  83) (i32.const  84)
        (i32.const  85) (i32.const  86) (i32.const  87) (i32.const  88) (i32.const  89) ;; 90
        (i32.const  90) (i32.const  91) (i32.const  92) (i32.const  93) (i32.const  94)
        (i32.const  95) (i32.const  96) (i32.const  97) (i32.const  98) (i32.const  99) ;; 100
        (i32.const 100) (i32.const 101) (i32.const 102) (i32.const 103) (i32.const 104)
        (i32.const 105) (i32.const 106) (i32.const 107) (i32.const 108) (i32.const 109) ;; 110
        (i32.const 110) (i32.const 111) (i32.const 112) (i32.const 113) (i32.const 114)
        (i32.const 115) (i32.const 116) (i32.const 117) (i32.const 118) (i32.const 119) ;; 120
        (i32.const 120) (i32.const 121) (i32.const 122) (i32.const 123) (i32.const 124)
        (i32.const 125) (i32.const 126) (i32.const 127) (i32.const 128) (i32.const 129) ;; 130
        (i32.const 130) (i32.const 131) (i32.const 132) (i32.const 133) (i32.const 134)
        (i32.const 135) (i32.const 136) (i32.const 137) (i32.const 138) (i32.const 139) ;; 140
        (i32.const 140) (i32.const 141) (i32.const 142) (i32.const 143) (i32.const 144)
        (i32.const 145) (i32.const 146) (i32.const 147) (i32.const 148) (i32.const 149) ;; 150
    )
    (i32.const   0) (i32.const   1) (i32.const   2) (i32.const   3) (i32.const   4)
    (i32.const   5) (i32.const   6) (i32.const   7) (i32.const   8) (i32.const   9) ;; 10
    (i32.const  10) (i32.const  11) (i32.const  12) (i32.const  13) (i32.const  14)
    (i32.const  15) (i32.const  16) (i32.const  17) (i32.const  18) (i32.const  19) ;; 20
    (i32.const  20) (i32.const  21) (i32.const  22) (i32.const  23) (i32.const  24)
    (i32.const  25) (i32.const  26) (i32.const  27) (i32.const  28) (i32.const  29) ;; 30
    (i32.const  30) (i32.const  31) (i32.const  32) (i32.const  33) (i32.const  34)
    (i32.const  35) (i32.const  36) (i32.const  37) (i32.const  38) (i32.const  39) ;; 40
    (i32.const  40) (i32.const  41) (i32.const  42) (i32.const  43) (i32.const  44)
    (i32.const  45) (i32.const  46) (i32.const  47) (i32.const  48) (i32.const  49) ;; 50
    (i32.const  50) (i32.const  51) (i32.const  52) (i32.const  53) (i32.const  54)
    (i32.const  55) (i32.const  56) (i32.const  57) (i32.const  58) (i32.const  59) ;; 60
    (i32.const  60) (i32.const  61) (i32.const  62) (i32.const  63) (i32.const  64)
    (i32.const  65) (i32.const  66) (i32.const  67) (i32.const  68) (i32.const  69) ;; 70
    (i32.const  70) (i32.const  71) (i32.const  72) (i32.const  73) (i32.const  74)
    (i32.const  75) (i32.const  76) (i32.const  77) (i32.const  78) (i32.const  79) ;; 80
    (i32.const  80) (i32.const  81) (i32.const  82) (i32.const  83) (i32.const  84)
    (i32.const  85) (i32.const  86) (i32.const  87) (i32.const  88) (i32.const  89) ;; 90
    (i32.const  90) (i32.const  91) (i32.const  92) (i32.const  93) (i32.const  94)
    (i32.const  95) (i32.const  96) (i32.const  97) (i32.const  98) (i32.const  99) ;; 100
    (i32.const 100) (i32.const 101) (i32.const 102) (i32.const 103) (i32.const 104)
    (i32.const 105) (i32.const 106) (i32.const 107) (i32.const 108) (i32.const 109) ;; 110
    (i32.const 110) (i32.const 111) (i32.const 112) (i32.const 113) (i32.const 114)
    (i32.const 115) (i32.const 116) (i32.const 117) (i32.const 118) (i32.const 119) ;; 120
    (i32.const 120) (i32.const 121) (i32.const 122) (i32.const 123) (i32.const 124)
    (i32.const 125) (i32.const 126) (i32.const 127) (i32.const 128) (i32.const 129) ;; 130
    (i32.const 130) (i32.const 131) (i32.const 132) (i32.const 133) (i32.const 134)
    (i32.const 135) (i32.const 136) (i32.const 137) (i32.const 138) (i32.const 139) ;; 140
    (i32.const 140) (i32.const 141) (i32.const 142) (i32.const 143) (i32.const 144)
    (i32.const 145) (i32.const 146) (i32.const 147) (i32.const 148) (i32.const 149) ;; 150
)

(assert_return
    (invoke "many-inout-reverse"
        (i32.const   0) (i32.const   1) (i32.const   2) (i32.const   3) (i32.const   4)
        (i32.const   5) (i32.const   6) (i32.const   7) (i32.const   8) (i32.const   9) ;; 10
        (i32.const  10) (i32.const  11) (i32.const  12) (i32.const  13) (i32.const  14)
        (i32.const  15) (i32.const  16) (i32.const  17) (i32.const  18) (i32.const  19) ;; 20
        (i32.const  20) (i32.const  21) (i32.const  22) (i32.const  23) (i32.const  24)
        (i32.const  25) (i32.const  26) (i32.const  27) (i32.const  28) (i32.const  29) ;; 30
        (i32.const  30) (i32.const  31) (i32.const  32) (i32.const  33) (i32.const  34)
        (i32.const  35) (i32.const  36) (i32.const  37) (i32.const  38) (i32.const  39) ;; 40
        (i32.const  40) (i32.const  41) (i32.const  42) (i32.const  43) (i32.const  44)
        (i32.const  45) (i32.const  46) (i32.const  47) (i32.const  48) (i32.const  49) ;; 50
        (i32.const  50) (i32.const  51) (i32.const  52) (i32.const  53) (i32.const  54)
        (i32.const  55) (i32.const  56) (i32.const  57) (i32.const  58) (i32.const  59) ;; 60
        (i32.const  60) (i32.const  61) (i32.const  62) (i32.const  63) (i32.const  64)
        (i32.const  65) (i32.const  66) (i32.const  67) (i32.const  68) (i32.const  69) ;; 70
        (i32.const  70) (i32.const  71) (i32.const  72) (i32.const  73) (i32.const  74)
        (i32.const  75) (i32.const  76) (i32.const  77) (i32.const  78) (i32.const  79) ;; 80
        (i32.const  80) (i32.const  81) (i32.const  82) (i32.const  83) (i32.const  84)
        (i32.const  85) (i32.const  86) (i32.const  87) (i32.const  88) (i32.const  89) ;; 90
        (i32.const  90) (i32.const  91) (i32.const  92) (i32.const  93) (i32.const  94)
        (i32.const  95) (i32.const  96) (i32.const  97) (i32.const  98) (i32.const  99) ;; 100
        (i32.const 100) (i32.const 101) (i32.const 102) (i32.const 103) (i32.const 104)
        (i32.const 105) (i32.const 106) (i32.const 107) (i32.const 108) (i32.const 109) ;; 110
        (i32.const 110) (i32.const 111) (i32.const 112) (i32.const 113) (i32.const 114)
        (i32.const 115) (i32.const 116) (i32.const 117) (i32.const 118) (i32.const 119) ;; 120
        (i32.const 120) (i32.const 121) (i32.const 122) (i32.const 123) (i32.const 124)
        (i32.const 125) (i32.const 126) (i32.const 127) (i32.const 128) (i32.const 129) ;; 130
        (i32.const 130) (i32.const 131) (i32.const 132) (i32.const 133) (i32.const 134)
        (i32.const 135) (i32.const 136) (i32.const 137) (i32.const 138) (i32.const 139) ;; 140
        (i32.const 140) (i32.const 141) (i32.const 142) (i32.const 143) (i32.const 144)
        (i32.const 145) (i32.const 146) (i32.const 147) (i32.const 148) (i32.const 149) ;; 150
    )
    (i32.const 149) (i32.const 148) (i32.const 147) (i32.const 146) (i32.const 145)
    (i32.const 144) (i32.const 143) (i32.const 142) (i32.const 141) (i32.const 140) ;; 10
    (i32.const 139) (i32.const 138) (i32.const 137) (i32.const 136) (i32.const 135)
    (i32.const 134) (i32.const 133) (i32.const 132) (i32.const 131) (i32.const 130) ;; 20
    (i32.const 129) (i32.const 128) (i32.const 127) (i32.const 126) (i32.const 125)
    (i32.const 124) (i32.const 123) (i32.const 122) (i32.const 121) (i32.const 120) ;; 30
    (i32.const 119) (i32.const 118) (i32.const 117) (i32.const 116) (i32.const 115)
    (i32.const 114) (i32.const 113) (i32.const 112) (i32.const 111) (i32.const 110) ;; 40
    (i32.const 109) (i32.const 108) (i32.const 107) (i32.const 106) (i32.const 105)
    (i32.const 104) (i32.const 103) (i32.const 102) (i32.const 101) (i32.const 100) ;; 50
    (i32.const  99) (i32.const  98) (i32.const  97) (i32.const  96) (i32.const  95)
    (i32.const  94) (i32.const  93) (i32.const  92) (i32.const  91) (i32.const  90) ;; 60
    (i32.const  89) (i32.const  88) (i32.const  87) (i32.const  86) (i32.const  85)
    (i32.const  84) (i32.const  83) (i32.const  82) (i32.const  81) (i32.const  80) ;; 70
    (i32.const  79) (i32.const  78) (i32.const  77) (i32.const  76) (i32.const  75)
    (i32.const  74) (i32.const  73) (i32.const  72) (i32.const  71) (i32.const  70) ;; 80
    (i32.const  69) (i32.const  68) (i32.const  67) (i32.const  66) (i32.const  65)
    (i32.const  64) (i32.const  63) (i32.const  62) (i32.const  61) (i32.const  60) ;; 90
    (i32.const  59) (i32.const  58) (i32.const  57) (i32.const  56) (i32.const  55)
    (i32.const  54) (i32.const  53) (i32.const  52) (i32.const  51) (i32.const  50) ;; 100
    (i32.const  49) (i32.const  48) (i32.const  47) (i32.const  46) (i32.const  45)
    (i32.const  44) (i32.const  43) (i32.const  42) (i32.const  41) (i32.const  40) ;; 110
    (i32.const  39) (i32.const  38) (i32.const  37) (i32.const  36) (i32.const  35)
    (i32.const  34) (i32.const  33) (i32.const  32) (i32.const  31) (i32.const  30) ;; 120
    (i32.const  29) (i32.const  28) (i32.const  27) (i32.const  26) (i32.const  25)
    (i32.const  24) (i32.const  23) (i32.const  22) (i32.const  21) (i32.const  20) ;; 130
    (i32.const  19) (i32.const  18) (i32.const  17) (i32.const  16) (i32.const  15)
    (i32.const  14) (i32.const  13) (i32.const  12) (i32.const  11) (i32.const  10) ;; 140
    (i32.const   9) (i32.const   8) (i32.const   7) (i32.const   6) (i32.const   5)
    (i32.const   4) (i32.const   3) (i32.const   2) (i32.const   1) (i32.const   0) ;; 150
)
