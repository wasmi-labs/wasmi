(module
  (type (;0;) (func (result i32)))
  (type (;1;) (func))
  (type (;2;) (func (param i32)))
  (type (;3;) (func (param i32) (result i32)))
  (type (;4;) (func (param i32 i32)))
  (import "env" "DYNAMICTOP_PTR" (global (;0;) i32))
  (import "env" "STACKTOP" (global (;1;) i32))
  (import "env" "STACK_MAX" (global (;2;) i32))
  (import "env" "enlargeMemory" (func (;0;) (type 0)))
  (import "env" "getTotalMemory" (func (;1;) (type 0)))
  (import "env" "abortOnCannotGrowMemory" (func (;2;) (type 0)))
  (import "env" "_abort" (func (;3;) (type 1)))
  (import "env" "___setErrNo" (func (;4;) (type 2)))
  (import "env" "memory" (memory (;0;) 256 256))
  (import "env" "table" (table (;0;) 0 0 anyfunc))
  (import "env" "memoryBase" (global (;3;) i32))
  (import "env" "tableBase" (global (;4;) i32))
  (func (;5;) (type 3) (param i32) (result i32)
    (local i32)
    block i32  ;; label = @1
      get_global 6
      set_local 1
      get_global 6
      get_local 0
      i32.add
      set_global 6
      get_global 6
      i32.const 15
      i32.add
      i32.const -16
      i32.and
      set_global 6
      get_local 1
    end)
  (func (;6;) (type 0) (result i32)
    get_global 6)
  (func (;7;) (type 2) (param i32)
    get_local 0
    set_global 6)
  (func (;8;) (type 4) (param i32 i32)
    block  ;; label = @1
      get_local 0
      set_global 6
      get_local 1
      set_global 7
    end)
  (func (;9;) (type 4) (param i32 i32)
    get_global 8
    i32.eqz
    if  ;; label = @1
      get_local 0
      set_global 8
      get_local 1
      set_global 9
    end)
  (func (;10;) (type 2) (param i32)
    get_local 0
    set_global 10)
  (func (;11;) (type 0) (result i32)
    get_global 10)
  (func (;12;) (type 0) (result i32)
    i32.const 144)
  (func (;13;) (type 0) (result i32)
    i32.const 1268)
  (func (;14;) (type 0) (result i32)
    call 15
    i32.const 64
    i32.add)
  (func (;15;) (type 0) (result i32)
    call 16)
  (func (;16;) (type 0) (result i32)
    i32.const 1024)
  (func (;17;) (type 3) (param i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
    block i32  ;; label = @1
      get_global 6
      set_local 13
      get_global 6
      i32.const 16
      i32.add
      set_global 6
      get_local 13
      set_local 16
      block  ;; label = @2
        get_local 0
        i32.const 245
        i32.lt_u
        if  ;; label = @3
          get_local 0
          i32.const 11
          i32.add
          i32.const -8
          i32.and
          set_local 2
          i32.const 1332
          i32.load
          tee_local 7
          get_local 0
          i32.const 11
          i32.lt_u
          if i32  ;; label = @4
            i32.const 16
            tee_local 2
          else
            get_local 2
          end
          i32.const 3
          i32.shr_u
          tee_local 0
          i32.shr_u
          tee_local 3
          i32.const 3
          i32.and
          if  ;; label = @4
            get_local 3
            i32.const 1
            i32.and
            i32.const 1
            i32.xor
            get_local 0
            i32.add
            tee_local 1
            i32.const 3
            i32.shl
            i32.const 1372
            i32.add
            tee_local 2
            i32.const 8
            i32.add
            tee_local 4
            i32.load
            tee_local 3
            i32.const 8
            i32.add
            tee_local 6
            i32.load
            set_local 0
            get_local 2
            get_local 0
            i32.eq
            if  ;; label = @5
              i32.const 1332
              get_local 7
              i32.const 1
              get_local 1
              i32.shl
              i32.const -1
              i32.xor
              i32.and
              i32.store
            else
              get_local 0
              i32.const 1348
              i32.load
              i32.lt_u
              if  ;; label = @6
                call 3
              end
              get_local 0
              i32.const 12
              i32.add
              tee_local 5
              i32.load
              get_local 3
              i32.eq
              if  ;; label = @6
                get_local 5
                get_local 2
                i32.store
                get_local 4
                get_local 0
                i32.store
              else
                call 3
              end
            end
            get_local 3
            get_local 1
            i32.const 3
            i32.shl
            tee_local 0
            i32.const 3
            i32.or
            i32.store offset=4
            get_local 3
            get_local 0
            i32.add
            i32.const 4
            i32.add
            tee_local 0
            get_local 0
            i32.load
            i32.const 1
            i32.or
            i32.store
            get_local 13
            set_global 6
            get_local 6
            return
          end
          get_local 2
          i32.const 1340
          i32.load
          tee_local 15
          i32.gt_u
          if  ;; label = @4
            get_local 3
            if  ;; label = @5
              get_local 3
              get_local 0
              i32.shl
              i32.const 2
              get_local 0
              i32.shl
              tee_local 0
              i32.const 0
              get_local 0
              i32.sub
              i32.or
              i32.and
              tee_local 0
              i32.const 0
              get_local 0
              i32.sub
              i32.and
              i32.const -1
              i32.add
              tee_local 3
              i32.const 12
              i32.shr_u
              i32.const 16
              i32.and
              set_local 0
              get_local 3
              get_local 0
              i32.shr_u
              tee_local 3
              i32.const 5
              i32.shr_u
              i32.const 8
              i32.and
              tee_local 4
              get_local 0
              i32.or
              get_local 3
              get_local 4
              i32.shr_u
              tee_local 0
              i32.const 2
              i32.shr_u
              i32.const 4
              i32.and
              tee_local 3
              i32.or
              get_local 0
              get_local 3
              i32.shr_u
              tee_local 0
              i32.const 1
              i32.shr_u
              i32.const 2
              i32.and
              tee_local 3
              i32.or
              get_local 0
              get_local 3
              i32.shr_u
              tee_local 0
              i32.const 1
              i32.shr_u
              i32.const 1
              i32.and
              tee_local 3
              i32.or
              get_local 0
              get_local 3
              i32.shr_u
              i32.add
              tee_local 4
              i32.const 3
              i32.shl
              i32.const 1372
              i32.add
              tee_local 5
              i32.const 8
              i32.add
              tee_local 8
              i32.load
              tee_local 3
              i32.const 8
              i32.add
              tee_local 10
              i32.load
              set_local 0
              get_local 5
              get_local 0
              i32.eq
              if  ;; label = @6
                i32.const 1332
                get_local 7
                i32.const 1
                get_local 4
                i32.shl
                i32.const -1
                i32.xor
                i32.and
                tee_local 1
                i32.store
              else
                get_local 0
                i32.const 1348
                i32.load
                i32.lt_u
                if  ;; label = @7
                  call 3
                end
                get_local 0
                i32.const 12
                i32.add
                tee_local 12
                i32.load
                get_local 3
                i32.eq
                if  ;; label = @7
                  get_local 12
                  get_local 5
                  i32.store
                  get_local 8
                  get_local 0
                  i32.store
                  get_local 7
                  set_local 1
                else
                  call 3
                end
              end
              get_local 3
              get_local 2
              i32.const 3
              i32.or
              i32.store offset=4
              get_local 3
              get_local 2
              i32.add
              tee_local 5
              get_local 4
              i32.const 3
              i32.shl
              get_local 2
              i32.sub
              tee_local 4
              i32.const 1
              i32.or
              i32.store offset=4
              get_local 5
              get_local 4
              i32.add
              get_local 4
              i32.store
              get_local 15
              if  ;; label = @6
                i32.const 1352
                i32.load
                set_local 2
                get_local 15
                i32.const 3
                i32.shr_u
                tee_local 3
                i32.const 3
                i32.shl
                i32.const 1372
                i32.add
                set_local 0
                get_local 1
                i32.const 1
                get_local 3
                i32.shl
                tee_local 3
                i32.and
                if  ;; label = @7
                  get_local 0
                  i32.const 8
                  i32.add
                  tee_local 3
                  i32.load
                  tee_local 1
                  i32.const 1348
                  i32.load
                  i32.lt_u
                  if  ;; label = @8
                    call 3
                  else
                    get_local 3
                    set_local 11
                    get_local 1
                    set_local 6
                  end
                else
                  i32.const 1332
                  get_local 1
                  get_local 3
                  i32.or
                  i32.store
                  get_local 0
                  i32.const 8
                  i32.add
                  set_local 11
                  get_local 0
                  set_local 6
                end
                get_local 11
                get_local 2
                i32.store
                get_local 6
                get_local 2
                i32.store offset=12
                get_local 2
                get_local 6
                i32.store offset=8
                get_local 2
                get_local 0
                i32.store offset=12
              end
              i32.const 1340
              get_local 4
              i32.store
              i32.const 1352
              get_local 5
              i32.store
              get_local 13
              set_global 6
              get_local 10
              return
            end
            i32.const 1336
            i32.load
            tee_local 11
            if  ;; label = @5
              get_local 11
              i32.const 0
              get_local 11
              i32.sub
              i32.and
              i32.const -1
              i32.add
              tee_local 3
              i32.const 12
              i32.shr_u
              i32.const 16
              i32.and
              set_local 0
              get_local 3
              get_local 0
              i32.shr_u
              tee_local 3
              i32.const 5
              i32.shr_u
              i32.const 8
              i32.and
              tee_local 1
              get_local 0
              i32.or
              get_local 3
              get_local 1
              i32.shr_u
              tee_local 0
              i32.const 2
              i32.shr_u
              i32.const 4
              i32.and
              tee_local 3
              i32.or
              get_local 0
              get_local 3
              i32.shr_u
              tee_local 0
              i32.const 1
              i32.shr_u
              i32.const 2
              i32.and
              tee_local 3
              i32.or
              get_local 0
              get_local 3
              i32.shr_u
              tee_local 0
              i32.const 1
              i32.shr_u
              i32.const 1
              i32.and
              tee_local 3
              i32.or
              get_local 0
              get_local 3
              i32.shr_u
              i32.add
              i32.const 2
              i32.shl
              i32.const 1636
              i32.add
              i32.load
              tee_local 1
              i32.load offset=4
              i32.const -8
              i32.and
              get_local 2
              i32.sub
              set_local 3
              get_local 1
              i32.const 16
              i32.add
              get_local 1
              i32.load offset=16
              i32.eqz
              i32.const 2
              i32.shl
              i32.add
              i32.load
              tee_local 0
              if  ;; label = @6
                loop  ;; label = @7
                  get_local 0
                  i32.load offset=4
                  i32.const -8
                  i32.and
                  get_local 2
                  i32.sub
                  tee_local 6
                  get_local 3
                  i32.lt_u
                  tee_local 8
                  if  ;; label = @8
                    get_local 6
                    set_local 3
                  end
                  get_local 8
                  if  ;; label = @8
                    get_local 0
                    set_local 1
                  end
                  get_local 0
                  i32.const 16
                  i32.add
                  get_local 0
                  i32.load offset=16
                  i32.eqz
                  i32.const 2
                  i32.shl
                  i32.add
                  i32.load
                  tee_local 0
                  br_if 0 (;@7;)
                  get_local 3
                  set_local 6
                end
              else
                get_local 3
                set_local 6
              end
              get_local 1
              i32.const 1348
              i32.load
              tee_local 16
              i32.lt_u
              if  ;; label = @6
                call 3
              end
              get_local 1
              get_local 1
              get_local 2
              i32.add
              tee_local 9
              i32.ge_u
              if  ;; label = @6
                call 3
              end
              get_local 1
              i32.load offset=24
              set_local 12
              block  ;; label = @6
                get_local 1
                i32.load offset=12
                tee_local 0
                get_local 1
                i32.eq
                if  ;; label = @7
                  get_local 1
                  i32.const 20
                  i32.add
                  tee_local 3
                  i32.load
                  tee_local 0
                  i32.eqz
                  if  ;; label = @8
                    get_local 1
                    i32.const 16
                    i32.add
                    tee_local 3
                    i32.load
                    tee_local 0
                    i32.eqz
                    if  ;; label = @9
                      i32.const 0
                      set_local 4
                      br 3 (;@6;)
                    end
                  end
                  loop  ;; label = @8
                    get_local 0
                    i32.const 20
                    i32.add
                    tee_local 8
                    i32.load
                    tee_local 10
                    if  ;; label = @9
                      get_local 10
                      set_local 0
                      get_local 8
                      set_local 3
                      br 1 (;@8;)
                    end
                    get_local 0
                    i32.const 16
                    i32.add
                    tee_local 8
                    i32.load
                    tee_local 10
                    if  ;; label = @9
                      get_local 10
                      set_local 0
                      get_local 8
                      set_local 3
                      br 1 (;@8;)
                    end
                  end
                  get_local 3
                  get_local 16
                  i32.lt_u
                  if  ;; label = @8
                    call 3
                  else
                    get_local 3
                    i32.const 0
                    i32.store
                    get_local 0
                    set_local 4
                  end
                else
                  get_local 1
                  i32.load offset=8
                  tee_local 3
                  get_local 16
                  i32.lt_u
                  if  ;; label = @8
                    call 3
                  end
                  get_local 3
                  i32.const 12
                  i32.add
                  tee_local 8
                  i32.load
                  get_local 1
                  i32.ne
                  if  ;; label = @8
                    call 3
                  end
                  get_local 0
                  i32.const 8
                  i32.add
                  tee_local 10
                  i32.load
                  get_local 1
                  i32.eq
                  if  ;; label = @8
                    get_local 8
                    get_local 0
                    i32.store
                    get_local 10
                    get_local 3
                    i32.store
                    get_local 0
                    set_local 4
                  else
                    call 3
                  end
                end
              end
              block  ;; label = @6
                get_local 12
                if  ;; label = @7
                  get_local 1
                  get_local 1
                  i32.load offset=28
                  tee_local 0
                  i32.const 2
                  i32.shl
                  i32.const 1636
                  i32.add
                  tee_local 3
                  i32.load
                  i32.eq
                  if  ;; label = @8
                    get_local 3
                    get_local 4
                    i32.store
                    get_local 4
                    i32.eqz
                    if  ;; label = @9
                      i32.const 1336
                      get_local 11
                      i32.const 1
                      get_local 0
                      i32.shl
                      i32.const -1
                      i32.xor
                      i32.and
                      i32.store
                      br 3 (;@6;)
                    end
                  else
                    get_local 12
                    i32.const 1348
                    i32.load
                    i32.lt_u
                    if  ;; label = @9
                      call 3
                    else
                      get_local 12
                      i32.const 16
                      i32.add
                      get_local 12
                      i32.load offset=16
                      get_local 1
                      i32.ne
                      i32.const 2
                      i32.shl
                      i32.add
                      get_local 4
                      i32.store
                      get_local 4
                      i32.eqz
                      br_if 3 (;@6;)
                    end
                  end
                  get_local 4
                  i32.const 1348
                  i32.load
                  tee_local 3
                  i32.lt_u
                  if  ;; label = @8
                    call 3
                  end
                  get_local 4
                  get_local 12
                  i32.store offset=24
                  get_local 1
                  i32.load offset=16
                  tee_local 0
                  if  ;; label = @8
                    get_local 0
                    get_local 3
                    i32.lt_u
                    if  ;; label = @9
                      call 3
                    else
                      get_local 4
                      get_local 0
                      i32.store offset=16
                      get_local 0
                      get_local 4
                      i32.store offset=24
                    end
                  end
                  get_local 1
                  i32.load offset=20
                  tee_local 0
                  if  ;; label = @8
                    get_local 0
                    i32.const 1348
                    i32.load
                    i32.lt_u
                    if  ;; label = @9
                      call 3
                    else
                      get_local 4
                      get_local 0
                      i32.store offset=20
                      get_local 0
                      get_local 4
                      i32.store offset=24
                    end
                  end
                end
              end
              get_local 6
              i32.const 16
              i32.lt_u
              if  ;; label = @6
                get_local 1
                get_local 6
                get_local 2
                i32.add
                tee_local 0
                i32.const 3
                i32.or
                i32.store offset=4
                get_local 1
                get_local 0
                i32.add
                i32.const 4
                i32.add
                tee_local 0
                get_local 0
                i32.load
                i32.const 1
                i32.or
                i32.store
              else
                get_local 1
                get_local 2
                i32.const 3
                i32.or
                i32.store offset=4
                get_local 9
                get_local 6
                i32.const 1
                i32.or
                i32.store offset=4
                get_local 9
                get_local 6
                i32.add
                get_local 6
                i32.store
                get_local 15
                if  ;; label = @7
                  i32.const 1352
                  i32.load
                  set_local 4
                  get_local 15
                  i32.const 3
                  i32.shr_u
                  tee_local 3
                  i32.const 3
                  i32.shl
                  i32.const 1372
                  i32.add
                  set_local 0
                  get_local 7
                  i32.const 1
                  get_local 3
                  i32.shl
                  tee_local 3
                  i32.and
                  if  ;; label = @8
                    get_local 0
                    i32.const 8
                    i32.add
                    tee_local 3
                    i32.load
                    tee_local 2
                    i32.const 1348
                    i32.load
                    i32.lt_u
                    if  ;; label = @9
                      call 3
                    else
                      get_local 3
                      set_local 14
                      get_local 2
                      set_local 5
                    end
                  else
                    i32.const 1332
                    get_local 7
                    get_local 3
                    i32.or
                    i32.store
                    get_local 0
                    i32.const 8
                    i32.add
                    set_local 14
                    get_local 0
                    set_local 5
                  end
                  get_local 14
                  get_local 4
                  i32.store
                  get_local 5
                  get_local 4
                  i32.store offset=12
                  get_local 4
                  get_local 5
                  i32.store offset=8
                  get_local 4
                  get_local 0
                  i32.store offset=12
                end
                i32.const 1340
                get_local 6
                i32.store
                i32.const 1352
                get_local 9
                i32.store
              end
              get_local 13
              set_global 6
              get_local 1
              i32.const 8
              i32.add
              return
            else
              get_local 2
              set_local 3
            end
          else
            get_local 2
            set_local 3
          end
        else
          get_local 0
          i32.const -65
          i32.gt_u
          if  ;; label = @4
            i32.const -1
            set_local 3
          else
            get_local 0
            i32.const 11
            i32.add
            tee_local 0
            i32.const -8
            i32.and
            set_local 4
            i32.const 1336
            i32.load
            tee_local 6
            if  ;; label = @5
              get_local 0
              i32.const 8
              i32.shr_u
              tee_local 0
              if i32  ;; label = @6
                get_local 4
                i32.const 16777215
                i32.gt_u
                if i32  ;; label = @7
                  i32.const 31
                else
                  get_local 4
                  i32.const 14
                  get_local 0
                  get_local 0
                  i32.const 1048320
                  i32.add
                  i32.const 16
                  i32.shr_u
                  i32.const 8
                  i32.and
                  tee_local 0
                  i32.shl
                  tee_local 1
                  i32.const 520192
                  i32.add
                  i32.const 16
                  i32.shr_u
                  i32.const 4
                  i32.and
                  tee_local 2
                  get_local 0
                  i32.or
                  get_local 1
                  get_local 2
                  i32.shl
                  tee_local 0
                  i32.const 245760
                  i32.add
                  i32.const 16
                  i32.shr_u
                  i32.const 2
                  i32.and
                  tee_local 1
                  i32.or
                  i32.sub
                  get_local 0
                  get_local 1
                  i32.shl
                  i32.const 15
                  i32.shr_u
                  i32.add
                  tee_local 0
                  i32.const 7
                  i32.add
                  i32.shr_u
                  i32.const 1
                  i32.and
                  get_local 0
                  i32.const 1
                  i32.shl
                  i32.or
                end
              else
                i32.const 0
              end
              set_local 17
              i32.const 0
              get_local 4
              i32.sub
              set_local 1
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    get_local 17
                    i32.const 2
                    i32.shl
                    i32.const 1636
                    i32.add
                    i32.load
                    tee_local 0
                    if  ;; label = @9
                      i32.const 25
                      get_local 17
                      i32.const 1
                      i32.shr_u
                      i32.sub
                      set_local 2
                      i32.const 0
                      set_local 5
                      get_local 4
                      get_local 17
                      i32.const 31
                      i32.eq
                      if i32  ;; label = @10
                        i32.const 0
                      else
                        get_local 2
                      end
                      i32.shl
                      set_local 11
                      i32.const 0
                      set_local 2
                      loop  ;; label = @10
                        get_local 0
                        i32.load offset=4
                        i32.const -8
                        i32.and
                        get_local 4
                        i32.sub
                        tee_local 14
                        get_local 1
                        i32.lt_u
                        if  ;; label = @11
                          get_local 14
                          if  ;; label = @12
                            get_local 14
                            set_local 1
                            get_local 0
                            set_local 2
                          else
                            i32.const 0
                            set_local 2
                            get_local 0
                            set_local 1
                            br 5 (;@7;)
                          end
                        end
                        get_local 0
                        i32.load offset=20
                        tee_local 18
                        i32.eqz
                        get_local 18
                        get_local 0
                        i32.const 16
                        i32.add
                        get_local 11
                        i32.const 31
                        i32.shr_u
                        i32.const 2
                        i32.shl
                        i32.add
                        i32.load
                        tee_local 14
                        i32.eq
                        i32.or
                        if i32  ;; label = @11
                          get_local 5
                        else
                          get_local 18
                        end
                        set_local 0
                        get_local 11
                        get_local 14
                        i32.eqz
                        tee_local 5
                        i32.const 1
                        i32.xor
                        i32.shl
                        set_local 11
                        get_local 5
                        br_if 2 (;@8;)
                        get_local 0
                        set_local 5
                        get_local 14
                        set_local 0
                        br 0 (;@10;)
                      end
                      unreachable
                    else
                      i32.const 0
                      set_local 0
                      i32.const 0
                      set_local 2
                    end
                  end
                  get_local 0
                  i32.eqz
                  get_local 2
                  i32.eqz
                  i32.and
                  if i32  ;; label = @8
                    get_local 6
                    i32.const 2
                    get_local 17
                    i32.shl
                    tee_local 0
                    i32.const 0
                    get_local 0
                    i32.sub
                    i32.or
                    i32.and
                    tee_local 0
                    i32.eqz
                    if  ;; label = @9
                      get_local 4
                      set_local 3
                      br 7 (;@2;)
                    end
                    get_local 0
                    i32.const 0
                    get_local 0
                    i32.sub
                    i32.and
                    i32.const -1
                    i32.add
                    tee_local 2
                    i32.const 12
                    i32.shr_u
                    i32.const 16
                    i32.and
                    set_local 0
                    get_local 2
                    get_local 0
                    i32.shr_u
                    tee_local 2
                    i32.const 5
                    i32.shr_u
                    i32.const 8
                    i32.and
                    tee_local 5
                    get_local 0
                    i32.or
                    get_local 2
                    get_local 5
                    i32.shr_u
                    tee_local 0
                    i32.const 2
                    i32.shr_u
                    i32.const 4
                    i32.and
                    tee_local 2
                    i32.or
                    get_local 0
                    get_local 2
                    i32.shr_u
                    tee_local 0
                    i32.const 1
                    i32.shr_u
                    i32.const 2
                    i32.and
                    tee_local 2
                    i32.or
                    get_local 0
                    get_local 2
                    i32.shr_u
                    tee_local 0
                    i32.const 1
                    i32.shr_u
                    i32.const 1
                    i32.and
                    tee_local 2
                    i32.or
                    get_local 0
                    get_local 2
                    i32.shr_u
                    i32.add
                    i32.const 2
                    i32.shl
                    i32.const 1636
                    i32.add
                    i32.load
                    set_local 5
                    i32.const 0
                  else
                    get_local 0
                    set_local 5
                    get_local 2
                  end
                  set_local 0
                  get_local 5
                  if  ;; label = @8
                    get_local 1
                    set_local 2
                    get_local 5
                    set_local 1
                    br 1 (;@7;)
                  else
                    get_local 1
                    set_local 5
                    get_local 0
                    set_local 2
                  end
                  br 1 (;@6;)
                end
                loop  ;; label = @7
                  get_local 1
                  i32.load offset=4
                  i32.const -8
                  i32.and
                  get_local 4
                  i32.sub
                  tee_local 5
                  get_local 2
                  i32.lt_u
                  tee_local 11
                  if  ;; label = @8
                    get_local 5
                    set_local 2
                  end
                  get_local 11
                  if  ;; label = @8
                    get_local 1
                    set_local 0
                  end
                  get_local 1
                  i32.const 16
                  i32.add
                  get_local 1
                  i32.load offset=16
                  i32.eqz
                  i32.const 2
                  i32.shl
                  i32.add
                  i32.load
                  tee_local 1
                  br_if 0 (;@7;)
                  get_local 2
                  set_local 5
                  get_local 0
                  set_local 2
                end
              end
              get_local 2
              if  ;; label = @6
                get_local 5
                i32.const 1340
                i32.load
                get_local 4
                i32.sub
                i32.lt_u
                if  ;; label = @7
                  get_local 2
                  i32.const 1348
                  i32.load
                  tee_local 14
                  i32.lt_u
                  if  ;; label = @8
                    call 3
                  end
                  get_local 2
                  get_local 2
                  get_local 4
                  i32.add
                  tee_local 9
                  i32.ge_u
                  if  ;; label = @8
                    call 3
                  end
                  get_local 2
                  i32.load offset=24
                  set_local 11
                  block  ;; label = @8
                    get_local 2
                    i32.load offset=12
                    tee_local 0
                    get_local 2
                    i32.eq
                    if  ;; label = @9
                      get_local 2
                      i32.const 20
                      i32.add
                      tee_local 1
                      i32.load
                      tee_local 0
                      i32.eqz
                      if  ;; label = @10
                        get_local 2
                        i32.const 16
                        i32.add
                        tee_local 1
                        i32.load
                        tee_local 0
                        i32.eqz
                        if  ;; label = @11
                          i32.const 0
                          set_local 8
                          br 3 (;@8;)
                        end
                      end
                      loop  ;; label = @10
                        get_local 0
                        i32.const 20
                        i32.add
                        tee_local 10
                        i32.load
                        tee_local 12
                        if  ;; label = @11
                          get_local 12
                          set_local 0
                          get_local 10
                          set_local 1
                          br 1 (;@10;)
                        end
                        get_local 0
                        i32.const 16
                        i32.add
                        tee_local 10
                        i32.load
                        tee_local 12
                        if  ;; label = @11
                          get_local 12
                          set_local 0
                          get_local 10
                          set_local 1
                          br 1 (;@10;)
                        end
                      end
                      get_local 1
                      get_local 14
                      i32.lt_u
                      if  ;; label = @10
                        call 3
                      else
                        get_local 1
                        i32.const 0
                        i32.store
                        get_local 0
                        set_local 8
                      end
                    else
                      get_local 2
                      i32.load offset=8
                      tee_local 1
                      get_local 14
                      i32.lt_u
                      if  ;; label = @10
                        call 3
                      end
                      get_local 1
                      i32.const 12
                      i32.add
                      tee_local 10
                      i32.load
                      get_local 2
                      i32.ne
                      if  ;; label = @10
                        call 3
                      end
                      get_local 0
                      i32.const 8
                      i32.add
                      tee_local 12
                      i32.load
                      get_local 2
                      i32.eq
                      if  ;; label = @10
                        get_local 10
                        get_local 0
                        i32.store
                        get_local 12
                        get_local 1
                        i32.store
                        get_local 0
                        set_local 8
                      else
                        call 3
                      end
                    end
                  end
                  block  ;; label = @8
                    get_local 11
                    if  ;; label = @9
                      get_local 2
                      get_local 2
                      i32.load offset=28
                      tee_local 0
                      i32.const 2
                      i32.shl
                      i32.const 1636
                      i32.add
                      tee_local 1
                      i32.load
                      i32.eq
                      if  ;; label = @10
                        get_local 1
                        get_local 8
                        i32.store
                        get_local 8
                        i32.eqz
                        if  ;; label = @11
                          i32.const 1336
                          get_local 6
                          i32.const 1
                          get_local 0
                          i32.shl
                          i32.const -1
                          i32.xor
                          i32.and
                          tee_local 3
                          i32.store
                          br 3 (;@8;)
                        end
                      else
                        get_local 11
                        i32.const 1348
                        i32.load
                        i32.lt_u
                        if  ;; label = @11
                          call 3
                        else
                          get_local 11
                          i32.const 16
                          i32.add
                          get_local 11
                          i32.load offset=16
                          get_local 2
                          i32.ne
                          i32.const 2
                          i32.shl
                          i32.add
                          get_local 8
                          i32.store
                          get_local 8
                          i32.eqz
                          if  ;; label = @12
                            get_local 6
                            set_local 3
                            br 4 (;@8;)
                          end
                        end
                      end
                      get_local 8
                      i32.const 1348
                      i32.load
                      tee_local 1
                      i32.lt_u
                      if  ;; label = @10
                        call 3
                      end
                      get_local 8
                      get_local 11
                      i32.store offset=24
                      get_local 2
                      i32.load offset=16
                      tee_local 0
                      if  ;; label = @10
                        get_local 0
                        get_local 1
                        i32.lt_u
                        if  ;; label = @11
                          call 3
                        else
                          get_local 8
                          get_local 0
                          i32.store offset=16
                          get_local 0
                          get_local 8
                          i32.store offset=24
                        end
                      end
                      get_local 2
                      i32.load offset=20
                      tee_local 0
                      if  ;; label = @10
                        get_local 0
                        i32.const 1348
                        i32.load
                        i32.lt_u
                        if  ;; label = @11
                          call 3
                        else
                          get_local 8
                          get_local 0
                          i32.store offset=20
                          get_local 0
                          get_local 8
                          i32.store offset=24
                          get_local 6
                          set_local 3
                        end
                      else
                        get_local 6
                        set_local 3
                      end
                    else
                      get_local 6
                      set_local 3
                    end
                  end
                  block  ;; label = @8
                    get_local 5
                    i32.const 16
                    i32.lt_u
                    if  ;; label = @9
                      get_local 2
                      get_local 5
                      get_local 4
                      i32.add
                      tee_local 0
                      i32.const 3
                      i32.or
                      i32.store offset=4
                      get_local 2
                      get_local 0
                      i32.add
                      i32.const 4
                      i32.add
                      tee_local 0
                      get_local 0
                      i32.load
                      i32.const 1
                      i32.or
                      i32.store
                    else
                      get_local 2
                      get_local 4
                      i32.const 3
                      i32.or
                      i32.store offset=4
                      get_local 9
                      get_local 5
                      i32.const 1
                      i32.or
                      i32.store offset=4
                      get_local 9
                      get_local 5
                      i32.add
                      get_local 5
                      i32.store
                      get_local 5
                      i32.const 3
                      i32.shr_u
                      set_local 1
                      get_local 5
                      i32.const 256
                      i32.lt_u
                      if  ;; label = @10
                        get_local 1
                        i32.const 3
                        i32.shl
                        i32.const 1372
                        i32.add
                        set_local 0
                        i32.const 1332
                        i32.load
                        tee_local 3
                        i32.const 1
                        get_local 1
                        i32.shl
                        tee_local 1
                        i32.and
                        if  ;; label = @11
                          get_local 0
                          i32.const 8
                          i32.add
                          tee_local 3
                          i32.load
                          tee_local 1
                          i32.const 1348
                          i32.load
                          i32.lt_u
                          if  ;; label = @12
                            call 3
                          else
                            get_local 3
                            set_local 15
                            get_local 1
                            set_local 7
                          end
                        else
                          i32.const 1332
                          get_local 3
                          get_local 1
                          i32.or
                          i32.store
                          get_local 0
                          i32.const 8
                          i32.add
                          set_local 15
                          get_local 0
                          set_local 7
                        end
                        get_local 15
                        get_local 9
                        i32.store
                        get_local 7
                        get_local 9
                        i32.store offset=12
                        get_local 9
                        get_local 7
                        i32.store offset=8
                        get_local 9
                        get_local 0
                        i32.store offset=12
                        br 2 (;@8;)
                      end
                      get_local 5
                      i32.const 8
                      i32.shr_u
                      tee_local 0
                      if i32  ;; label = @10
                        get_local 5
                        i32.const 16777215
                        i32.gt_u
                        if i32  ;; label = @11
                          i32.const 31
                        else
                          get_local 5
                          i32.const 14
                          get_local 0
                          get_local 0
                          i32.const 1048320
                          i32.add
                          i32.const 16
                          i32.shr_u
                          i32.const 8
                          i32.and
                          tee_local 0
                          i32.shl
                          tee_local 1
                          i32.const 520192
                          i32.add
                          i32.const 16
                          i32.shr_u
                          i32.const 4
                          i32.and
                          tee_local 4
                          get_local 0
                          i32.or
                          get_local 1
                          get_local 4
                          i32.shl
                          tee_local 0
                          i32.const 245760
                          i32.add
                          i32.const 16
                          i32.shr_u
                          i32.const 2
                          i32.and
                          tee_local 1
                          i32.or
                          i32.sub
                          get_local 0
                          get_local 1
                          i32.shl
                          i32.const 15
                          i32.shr_u
                          i32.add
                          tee_local 0
                          i32.const 7
                          i32.add
                          i32.shr_u
                          i32.const 1
                          i32.and
                          get_local 0
                          i32.const 1
                          i32.shl
                          i32.or
                        end
                      else
                        i32.const 0
                      end
                      tee_local 1
                      i32.const 2
                      i32.shl
                      i32.const 1636
                      i32.add
                      set_local 0
                      get_local 9
                      get_local 1
                      i32.store offset=28
                      get_local 9
                      i32.const 16
                      i32.add
                      tee_local 4
                      i32.const 0
                      i32.store offset=4
                      get_local 4
                      i32.const 0
                      i32.store
                      get_local 3
                      i32.const 1
                      get_local 1
                      i32.shl
                      tee_local 4
                      i32.and
                      i32.eqz
                      if  ;; label = @10
                        i32.const 1336
                        get_local 3
                        get_local 4
                        i32.or
                        i32.store
                        get_local 0
                        get_local 9
                        i32.store
                        get_local 9
                        get_local 0
                        i32.store offset=24
                        get_local 9
                        get_local 9
                        i32.store offset=12
                        get_local 9
                        get_local 9
                        i32.store offset=8
                        br 2 (;@8;)
                      end
                      get_local 0
                      i32.load
                      set_local 0
                      i32.const 25
                      get_local 1
                      i32.const 1
                      i32.shr_u
                      i32.sub
                      set_local 3
                      get_local 5
                      get_local 1
                      i32.const 31
                      i32.eq
                      if i32  ;; label = @10
                        i32.const 0
                      else
                        get_local 3
                      end
                      i32.shl
                      set_local 3
                      block  ;; label = @10
                        block  ;; label = @11
                          block  ;; label = @12
                            loop  ;; label = @13
                              get_local 0
                              i32.load offset=4
                              i32.const -8
                              i32.and
                              get_local 5
                              i32.eq
                              br_if 2 (;@11;)
                              get_local 3
                              i32.const 1
                              i32.shl
                              set_local 1
                              get_local 0
                              i32.const 16
                              i32.add
                              get_local 3
                              i32.const 31
                              i32.shr_u
                              i32.const 2
                              i32.shl
                              i32.add
                              tee_local 3
                              i32.load
                              tee_local 4
                              i32.eqz
                              br_if 1 (;@12;)
                              get_local 1
                              set_local 3
                              get_local 4
                              set_local 0
                              br 0 (;@13;)
                            end
                            unreachable
                          end
                          get_local 3
                          i32.const 1348
                          i32.load
                          i32.lt_u
                          if  ;; label = @12
                            call 3
                          else
                            get_local 3
                            get_local 9
                            i32.store
                            get_local 9
                            get_local 0
                            i32.store offset=24
                            get_local 9
                            get_local 9
                            i32.store offset=12
                            get_local 9
                            get_local 9
                            i32.store offset=8
                            br 4 (;@8;)
                          end
                          br 1 (;@10;)
                        end
                        get_local 0
                        i32.const 8
                        i32.add
                        tee_local 1
                        i32.load
                        tee_local 3
                        i32.const 1348
                        i32.load
                        tee_local 4
                        i32.ge_u
                        get_local 0
                        get_local 4
                        i32.ge_u
                        i32.and
                        if  ;; label = @11
                          get_local 3
                          get_local 9
                          i32.store offset=12
                          get_local 1
                          get_local 9
                          i32.store
                          get_local 9
                          get_local 3
                          i32.store offset=8
                          get_local 9
                          get_local 0
                          i32.store offset=12
                          get_local 9
                          i32.const 0
                          i32.store offset=24
                        else
                          call 3
                        end
                      end
                    end
                  end
                  get_local 13
                  set_global 6
                  get_local 2
                  i32.const 8
                  i32.add
                  return
                else
                  get_local 4
                  set_local 3
                end
              else
                get_local 4
                set_local 3
              end
            else
              get_local 4
              set_local 3
            end
          end
        end
      end
      i32.const 1340
      i32.load
      tee_local 2
      get_local 3
      i32.ge_u
      if  ;; label = @2
        i32.const 1352
        i32.load
        set_local 0
        get_local 2
        get_local 3
        i32.sub
        tee_local 1
        i32.const 15
        i32.gt_u
        if  ;; label = @3
          i32.const 1352
          get_local 0
          get_local 3
          i32.add
          tee_local 2
          i32.store
          i32.const 1340
          get_local 1
          i32.store
          get_local 2
          get_local 1
          i32.const 1
          i32.or
          i32.store offset=4
          get_local 2
          get_local 1
          i32.add
          get_local 1
          i32.store
          get_local 0
          get_local 3
          i32.const 3
          i32.or
          i32.store offset=4
        else
          i32.const 1340
          i32.const 0
          i32.store
          i32.const 1352
          i32.const 0
          i32.store
          get_local 0
          get_local 2
          i32.const 3
          i32.or
          i32.store offset=4
          get_local 0
          get_local 2
          i32.add
          i32.const 4
          i32.add
          tee_local 3
          get_local 3
          i32.load
          i32.const 1
          i32.or
          i32.store
        end
        get_local 13
        set_global 6
        get_local 0
        i32.const 8
        i32.add
        return
      end
      i32.const 1344
      i32.load
      tee_local 1
      get_local 3
      i32.gt_u
      if  ;; label = @2
        i32.const 1344
        get_local 1
        get_local 3
        i32.sub
        tee_local 1
        i32.store
        i32.const 1356
        i32.const 1356
        i32.load
        tee_local 0
        get_local 3
        i32.add
        tee_local 2
        i32.store
        get_local 2
        get_local 1
        i32.const 1
        i32.or
        i32.store offset=4
        get_local 0
        get_local 3
        i32.const 3
        i32.or
        i32.store offset=4
        get_local 13
        set_global 6
        get_local 0
        i32.const 8
        i32.add
        return
      end
      i32.const 1804
      i32.load
      if i32  ;; label = @2
        i32.const 1812
        i32.load
      else
        i32.const 1812
        i32.const 4096
        i32.store
        i32.const 1808
        i32.const 4096
        i32.store
        i32.const 1816
        i32.const -1
        i32.store
        i32.const 1820
        i32.const -1
        i32.store
        i32.const 1824
        i32.const 0
        i32.store
        i32.const 1776
        i32.const 0
        i32.store
        get_local 16
        get_local 16
        i32.const -16
        i32.and
        i32.const 1431655768
        i32.xor
        tee_local 0
        i32.store
        i32.const 1804
        get_local 0
        i32.store
        i32.const 4096
      end
      tee_local 0
      get_local 3
      i32.const 47
      i32.add
      tee_local 6
      i32.add
      tee_local 5
      i32.const 0
      get_local 0
      i32.sub
      tee_local 8
      i32.and
      tee_local 4
      get_local 3
      i32.le_u
      if  ;; label = @2
        get_local 13
        set_global 6
        i32.const 0
        return
      end
      i32.const 1772
      i32.load
      tee_local 0
      if  ;; label = @2
        i32.const 1764
        i32.load
        tee_local 2
        get_local 4
        i32.add
        tee_local 7
        get_local 2
        i32.le_u
        get_local 7
        get_local 0
        i32.gt_u
        i32.or
        if  ;; label = @3
          get_local 13
          set_global 6
          i32.const 0
          return
        end
      end
      get_local 3
      i32.const 48
      i32.add
      set_local 7
      block  ;; label = @2
        block  ;; label = @3
          i32.const 1776
          i32.load
          i32.const 4
          i32.and
          if  ;; label = @4
            i32.const 0
            set_local 1
          else
            block  ;; label = @5
              block  ;; label = @6
                block  ;; label = @7
                  i32.const 1356
                  i32.load
                  tee_local 0
                  i32.eqz
                  br_if 0 (;@7;)
                  i32.const 1780
                  set_local 2
                  loop  ;; label = @8
                    block  ;; label = @9
                      get_local 2
                      i32.load
                      tee_local 11
                      get_local 0
                      i32.le_u
                      if  ;; label = @10
                        get_local 11
                        get_local 2
                        i32.const 4
                        i32.add
                        tee_local 11
                        i32.load
                        i32.add
                        get_local 0
                        i32.gt_u
                        br_if 1 (;@9;)
                      end
                      get_local 2
                      i32.load offset=8
                      tee_local 2
                      br_if 1 (;@8;)
                      br 2 (;@7;)
                    end
                  end
                  get_local 5
                  get_local 1
                  i32.sub
                  get_local 8
                  i32.and
                  tee_local 1
                  i32.const 2147483647
                  i32.lt_u
                  if  ;; label = @8
                    get_local 1
                    call 20
                    tee_local 0
                    get_local 2
                    i32.load
                    get_local 11
                    i32.load
                    i32.add
                    i32.eq
                    if  ;; label = @9
                      get_local 0
                      i32.const -1
                      i32.ne
                      br_if 6 (;@3;)
                    else
                      br 3 (;@6;)
                    end
                  else
                    i32.const 0
                    set_local 1
                  end
                  br 2 (;@5;)
                end
                i32.const 0
                call 20
                tee_local 0
                i32.const -1
                i32.eq
                if  ;; label = @7
                  i32.const 0
                  set_local 1
                else
                  i32.const 1808
                  i32.load
                  tee_local 2
                  i32.const -1
                  i32.add
                  tee_local 5
                  get_local 0
                  tee_local 1
                  i32.add
                  i32.const 0
                  get_local 2
                  i32.sub
                  i32.and
                  get_local 1
                  i32.sub
                  set_local 2
                  get_local 5
                  get_local 1
                  i32.and
                  if i32  ;; label = @8
                    get_local 2
                  else
                    i32.const 0
                  end
                  get_local 4
                  i32.add
                  tee_local 1
                  i32.const 1764
                  i32.load
                  tee_local 5
                  i32.add
                  set_local 2
                  get_local 1
                  get_local 3
                  i32.gt_u
                  get_local 1
                  i32.const 2147483647
                  i32.lt_u
                  i32.and
                  if  ;; label = @8
                    i32.const 1772
                    i32.load
                    tee_local 8
                    if  ;; label = @9
                      get_local 2
                      get_local 5
                      i32.le_u
                      get_local 2
                      get_local 8
                      i32.gt_u
                      i32.or
                      if  ;; label = @10
                        i32.const 0
                        set_local 1
                        br 5 (;@5;)
                      end
                    end
                    get_local 1
                    call 20
                    tee_local 2
                    get_local 0
                    i32.eq
                    br_if 5 (;@3;)
                    get_local 2
                    set_local 0
                    br 2 (;@6;)
                  else
                    i32.const 0
                    set_local 1
                  end
                end
                br 1 (;@5;)
              end
              get_local 7
              get_local 1
              i32.gt_u
              get_local 1
              i32.const 2147483647
              i32.lt_u
              get_local 0
              i32.const -1
              i32.ne
              i32.and
              i32.and
              i32.eqz
              if  ;; label = @6
                get_local 0
                i32.const -1
                i32.eq
                if  ;; label = @7
                  i32.const 0
                  set_local 1
                  br 2 (;@5;)
                else
                  br 4 (;@3;)
                end
                unreachable
              end
              get_local 6
              get_local 1
              i32.sub
              i32.const 1812
              i32.load
              tee_local 2
              i32.add
              i32.const 0
              get_local 2
              i32.sub
              i32.and
              tee_local 2
              i32.const 2147483647
              i32.ge_u
              br_if 2 (;@3;)
              i32.const 0
              get_local 1
              i32.sub
              set_local 6
              get_local 2
              call 20
              i32.const -1
              i32.eq
              if  ;; label = @6
                get_local 6
                call 20
                drop
                i32.const 0
                set_local 1
              else
                get_local 2
                get_local 1
                i32.add
                set_local 1
                br 3 (;@3;)
              end
            end
            i32.const 1776
            i32.const 1776
            i32.load
            i32.const 4
            i32.or
            i32.store
          end
          get_local 4
          i32.const 2147483647
          i32.lt_u
          if  ;; label = @4
            get_local 4
            call 20
            tee_local 0
            i32.const 0
            call 20
            tee_local 2
            i32.lt_u
            get_local 0
            i32.const -1
            i32.ne
            get_local 2
            i32.const -1
            i32.ne
            i32.and
            i32.and
            set_local 4
            get_local 2
            get_local 0
            i32.sub
            tee_local 2
            get_local 3
            i32.const 40
            i32.add
            i32.gt_u
            tee_local 6
            if  ;; label = @5
              get_local 2
              set_local 1
            end
            get_local 0
            i32.const -1
            i32.eq
            get_local 6
            i32.const 1
            i32.xor
            i32.or
            get_local 4
            i32.const 1
            i32.xor
            i32.or
            i32.eqz
            br_if 1 (;@3;)
          end
          br 1 (;@2;)
        end
        i32.const 1764
        i32.const 1764
        i32.load
        get_local 1
        i32.add
        tee_local 2
        i32.store
        get_local 2
        i32.const 1768
        i32.load
        i32.gt_u
        if  ;; label = @3
          i32.const 1768
          get_local 2
          i32.store
        end
        block  ;; label = @3
          i32.const 1356
          i32.load
          tee_local 6
          if  ;; label = @4
            i32.const 1780
            set_local 2
            block  ;; label = @5
              block  ;; label = @6
                loop  ;; label = @7
                  get_local 0
                  get_local 2
                  i32.load
                  tee_local 4
                  get_local 2
                  i32.const 4
                  i32.add
                  tee_local 5
                  i32.load
                  tee_local 8
                  i32.add
                  i32.eq
                  br_if 1 (;@6;)
                  get_local 2
                  i32.load offset=8
                  tee_local 2
                  br_if 0 (;@7;)
                end
                br 1 (;@5;)
              end
              get_local 2
              i32.load offset=12
              i32.const 8
              i32.and
              i32.eqz
              if  ;; label = @6
                get_local 6
                get_local 0
                i32.lt_u
                get_local 6
                get_local 4
                i32.ge_u
                i32.and
                if  ;; label = @7
                  get_local 5
                  get_local 8
                  get_local 1
                  i32.add
                  i32.store
                  i32.const 1344
                  i32.load
                  set_local 4
                  i32.const 0
                  get_local 6
                  i32.const 8
                  i32.add
                  tee_local 2
                  i32.sub
                  i32.const 7
                  i32.and
                  set_local 0
                  i32.const 1356
                  get_local 6
                  get_local 2
                  i32.const 7
                  i32.and
                  if i32  ;; label = @8
                    get_local 0
                  else
                    i32.const 0
                    tee_local 0
                  end
                  i32.add
                  tee_local 2
                  i32.store
                  i32.const 1344
                  get_local 4
                  get_local 1
                  get_local 0
                  i32.sub
                  i32.add
                  tee_local 0
                  i32.store
                  get_local 2
                  get_local 0
                  i32.const 1
                  i32.or
                  i32.store offset=4
                  get_local 2
                  get_local 0
                  i32.add
                  i32.const 40
                  i32.store offset=4
                  i32.const 1360
                  i32.const 1820
                  i32.load
                  i32.store
                  br 4 (;@3;)
                end
              end
            end
            get_local 0
            i32.const 1348
            i32.load
            tee_local 2
            i32.lt_u
            if  ;; label = @5
              i32.const 1348
              get_local 0
              i32.store
              get_local 0
              set_local 2
            end
            get_local 0
            get_local 1
            i32.add
            set_local 5
            i32.const 1780
            set_local 4
            block  ;; label = @5
              block  ;; label = @6
                loop  ;; label = @7
                  get_local 4
                  i32.load
                  get_local 5
                  i32.eq
                  br_if 1 (;@6;)
                  get_local 4
                  i32.load offset=8
                  tee_local 4
                  br_if 0 (;@7;)
                end
                br 1 (;@5;)
              end
              get_local 4
              i32.load offset=12
              i32.const 8
              i32.and
              i32.eqz
              if  ;; label = @6
                get_local 4
                get_local 0
                i32.store
                get_local 4
                i32.const 4
                i32.add
                tee_local 4
                get_local 4
                i32.load
                get_local 1
                i32.add
                i32.store
                i32.const 0
                get_local 0
                i32.const 8
                i32.add
                tee_local 1
                i32.sub
                i32.const 7
                i32.and
                set_local 4
                i32.const 0
                get_local 5
                i32.const 8
                i32.add
                tee_local 8
                i32.sub
                i32.const 7
                i32.and
                set_local 11
                get_local 0
                get_local 1
                i32.const 7
                i32.and
                if i32  ;; label = @7
                  get_local 4
                else
                  i32.const 0
                end
                i32.add
                tee_local 9
                get_local 3
                i32.add
                set_local 7
                get_local 5
                get_local 8
                i32.const 7
                i32.and
                if i32  ;; label = @7
                  get_local 11
                else
                  i32.const 0
                end
                i32.add
                tee_local 5
                get_local 9
                i32.sub
                get_local 3
                i32.sub
                set_local 8
                get_local 9
                get_local 3
                i32.const 3
                i32.or
                i32.store offset=4
                block  ;; label = @7
                  get_local 5
                  get_local 6
                  i32.eq
                  if  ;; label = @8
                    i32.const 1344
                    i32.const 1344
                    i32.load
                    get_local 8
                    i32.add
                    tee_local 0
                    i32.store
                    i32.const 1356
                    get_local 7
                    i32.store
                    get_local 7
                    get_local 0
                    i32.const 1
                    i32.or
                    i32.store offset=4
                  else
                    get_local 5
                    i32.const 1352
                    i32.load
                    i32.eq
                    if  ;; label = @9
                      i32.const 1340
                      i32.const 1340
                      i32.load
                      get_local 8
                      i32.add
                      tee_local 0
                      i32.store
                      i32.const 1352
                      get_local 7
                      i32.store
                      get_local 7
                      get_local 0
                      i32.const 1
                      i32.or
                      i32.store offset=4
                      get_local 7
                      get_local 0
                      i32.add
                      get_local 0
                      i32.store
                      br 2 (;@7;)
                    end
                    get_local 5
                    i32.load offset=4
                    tee_local 0
                    i32.const 3
                    i32.and
                    i32.const 1
                    i32.eq
                    if i32  ;; label = @9
                      get_local 0
                      i32.const -8
                      i32.and
                      set_local 11
                      get_local 0
                      i32.const 3
                      i32.shr_u
                      set_local 4
                      block  ;; label = @10
                        get_local 0
                        i32.const 256
                        i32.lt_u
                        if  ;; label = @11
                          get_local 5
                          i32.load offset=12
                          set_local 3
                          block  ;; label = @12
                            get_local 5
                            i32.load offset=8
                            tee_local 1
                            get_local 4
                            i32.const 3
                            i32.shl
                            i32.const 1372
                            i32.add
                            tee_local 0
                            i32.ne
                            if  ;; label = @13
                              get_local 1
                              get_local 2
                              i32.lt_u
                              if  ;; label = @14
                                call 3
                              end
                              get_local 1
                              i32.load offset=12
                              get_local 5
                              i32.eq
                              br_if 1 (;@12;)
                              call 3
                            end
                          end
                          get_local 3
                          get_local 1
                          i32.eq
                          if  ;; label = @12
                            i32.const 1332
                            i32.const 1332
                            i32.load
                            i32.const 1
                            get_local 4
                            i32.shl
                            i32.const -1
                            i32.xor
                            i32.and
                            i32.store
                            br 2 (;@10;)
                          end
                          block  ;; label = @12
                            get_local 3
                            get_local 0
                            i32.eq
                            if  ;; label = @13
                              get_local 3
                              i32.const 8
                              i32.add
                              set_local 19
                            else
                              get_local 3
                              get_local 2
                              i32.lt_u
                              if  ;; label = @14
                                call 3
                              end
                              get_local 3
                              i32.const 8
                              i32.add
                              tee_local 0
                              i32.load
                              get_local 5
                              i32.eq
                              if  ;; label = @14
                                get_local 0
                                set_local 19
                                br 2 (;@12;)
                              end
                              call 3
                            end
                          end
                          get_local 1
                          get_local 3
                          i32.store offset=12
                          get_local 19
                          get_local 1
                          i32.store
                        else
                          get_local 5
                          i32.load offset=24
                          set_local 6
                          block  ;; label = @12
                            get_local 5
                            i32.load offset=12
                            tee_local 0
                            get_local 5
                            i32.eq
                            if  ;; label = @13
                              get_local 5
                              i32.const 16
                              i32.add
                              tee_local 3
                              i32.const 4
                              i32.add
                              tee_local 1
                              i32.load
                              tee_local 0
                              if  ;; label = @14
                                get_local 1
                                set_local 3
                              else
                                get_local 3
                                i32.load
                                tee_local 0
                                i32.eqz
                                if  ;; label = @15
                                  i32.const 0
                                  set_local 10
                                  br 3 (;@12;)
                                end
                              end
                              loop  ;; label = @14
                                get_local 0
                                i32.const 20
                                i32.add
                                tee_local 1
                                i32.load
                                tee_local 4
                                if  ;; label = @15
                                  get_local 4
                                  set_local 0
                                  get_local 1
                                  set_local 3
                                  br 1 (;@14;)
                                end
                                get_local 0
                                i32.const 16
                                i32.add
                                tee_local 1
                                i32.load
                                tee_local 4
                                if  ;; label = @15
                                  get_local 4
                                  set_local 0
                                  get_local 1
                                  set_local 3
                                  br 1 (;@14;)
                                end
                              end
                              get_local 3
                              get_local 2
                              i32.lt_u
                              if  ;; label = @14
                                call 3
                              else
                                get_local 3
                                i32.const 0
                                i32.store
                                get_local 0
                                set_local 10
                              end
                            else
                              get_local 5
                              i32.load offset=8
                              tee_local 3
                              get_local 2
                              i32.lt_u
                              if  ;; label = @14
                                call 3
                              end
                              get_local 3
                              i32.const 12
                              i32.add
                              tee_local 1
                              i32.load
                              get_local 5
                              i32.ne
                              if  ;; label = @14
                                call 3
                              end
                              get_local 0
                              i32.const 8
                              i32.add
                              tee_local 2
                              i32.load
                              get_local 5
                              i32.eq
                              if  ;; label = @14
                                get_local 1
                                get_local 0
                                i32.store
                                get_local 2
                                get_local 3
                                i32.store
                                get_local 0
                                set_local 10
                              else
                                call 3
                              end
                            end
                          end
                          get_local 6
                          i32.eqz
                          br_if 1 (;@10;)
                          block  ;; label = @12
                            get_local 5
                            get_local 5
                            i32.load offset=28
                            tee_local 0
                            i32.const 2
                            i32.shl
                            i32.const 1636
                            i32.add
                            tee_local 3
                            i32.load
                            i32.eq
                            if  ;; label = @13
                              get_local 3
                              get_local 10
                              i32.store
                              get_local 10
                              br_if 1 (;@12;)
                              i32.const 1336
                              i32.const 1336
                              i32.load
                              i32.const 1
                              get_local 0
                              i32.shl
                              i32.const -1
                              i32.xor
                              i32.and
                              i32.store
                              br 3 (;@10;)
                            else
                              get_local 6
                              i32.const 1348
                              i32.load
                              i32.lt_u
                              if  ;; label = @14
                                call 3
                              else
                                get_local 6
                                i32.const 16
                                i32.add
                                get_local 6
                                i32.load offset=16
                                get_local 5
                                i32.ne
                                i32.const 2
                                i32.shl
                                i32.add
                                get_local 10
                                i32.store
                                get_local 10
                                i32.eqz
                                br_if 4 (;@10;)
                              end
                            end
                          end
                          get_local 10
                          i32.const 1348
                          i32.load
                          tee_local 3
                          i32.lt_u
                          if  ;; label = @12
                            call 3
                          end
                          get_local 10
                          get_local 6
                          i32.store offset=24
                          get_local 5
                          i32.const 16
                          i32.add
                          tee_local 1
                          i32.load
                          tee_local 0
                          if  ;; label = @12
                            get_local 0
                            get_local 3
                            i32.lt_u
                            if  ;; label = @13
                              call 3
                            else
                              get_local 10
                              get_local 0
                              i32.store offset=16
                              get_local 0
                              get_local 10
                              i32.store offset=24
                            end
                          end
                          get_local 1
                          i32.load offset=4
                          tee_local 0
                          i32.eqz
                          br_if 1 (;@10;)
                          get_local 0
                          i32.const 1348
                          i32.load
                          i32.lt_u
                          if  ;; label = @12
                            call 3
                          else
                            get_local 10
                            get_local 0
                            i32.store offset=20
                            get_local 0
                            get_local 10
                            i32.store offset=24
                          end
                        end
                      end
                      get_local 5
                      get_local 11
                      i32.add
                      set_local 5
                      get_local 11
                      get_local 8
                      i32.add
                    else
                      get_local 8
                    end
                    set_local 4
                    get_local 5
                    i32.const 4
                    i32.add
                    tee_local 0
                    get_local 0
                    i32.load
                    i32.const -2
                    i32.and
                    i32.store
                    get_local 7
                    get_local 4
                    i32.const 1
                    i32.or
                    i32.store offset=4
                    get_local 7
                    get_local 4
                    i32.add
                    get_local 4
                    i32.store
                    get_local 4
                    i32.const 3
                    i32.shr_u
                    set_local 3
                    get_local 4
                    i32.const 256
                    i32.lt_u
                    if  ;; label = @9
                      get_local 3
                      i32.const 3
                      i32.shl
                      i32.const 1372
                      i32.add
                      set_local 0
                      block  ;; label = @10
                        i32.const 1332
                        i32.load
                        tee_local 1
                        i32.const 1
                        get_local 3
                        i32.shl
                        tee_local 3
                        i32.and
                        if  ;; label = @11
                          get_local 0
                          i32.const 8
                          i32.add
                          tee_local 3
                          i32.load
                          tee_local 1
                          i32.const 1348
                          i32.load
                          i32.ge_u
                          if  ;; label = @12
                            get_local 3
                            set_local 20
                            get_local 1
                            set_local 12
                            br 2 (;@10;)
                          end
                          call 3
                        else
                          i32.const 1332
                          get_local 1
                          get_local 3
                          i32.or
                          i32.store
                          get_local 0
                          i32.const 8
                          i32.add
                          set_local 20
                          get_local 0
                          set_local 12
                        end
                      end
                      get_local 20
                      get_local 7
                      i32.store
                      get_local 12
                      get_local 7
                      i32.store offset=12
                      get_local 7
                      get_local 12
                      i32.store offset=8
                      get_local 7
                      get_local 0
                      i32.store offset=12
                      br 2 (;@7;)
                    end
                    block i32  ;; label = @9
                      get_local 4
                      i32.const 8
                      i32.shr_u
                      tee_local 0
                      if i32  ;; label = @10
                        i32.const 31
                        get_local 4
                        i32.const 16777215
                        i32.gt_u
                        br_if 1 (;@9;)
                        drop
                        get_local 4
                        i32.const 14
                        get_local 0
                        get_local 0
                        i32.const 1048320
                        i32.add
                        i32.const 16
                        i32.shr_u
                        i32.const 8
                        i32.and
                        tee_local 0
                        i32.shl
                        tee_local 3
                        i32.const 520192
                        i32.add
                        i32.const 16
                        i32.shr_u
                        i32.const 4
                        i32.and
                        tee_local 1
                        get_local 0
                        i32.or
                        get_local 3
                        get_local 1
                        i32.shl
                        tee_local 0
                        i32.const 245760
                        i32.add
                        i32.const 16
                        i32.shr_u
                        i32.const 2
                        i32.and
                        tee_local 3
                        i32.or
                        i32.sub
                        get_local 0
                        get_local 3
                        i32.shl
                        i32.const 15
                        i32.shr_u
                        i32.add
                        tee_local 0
                        i32.const 7
                        i32.add
                        i32.shr_u
                        i32.const 1
                        i32.and
                        get_local 0
                        i32.const 1
                        i32.shl
                        i32.or
                      else
                        i32.const 0
                      end
                    end
                    tee_local 3
                    i32.const 2
                    i32.shl
                    i32.const 1636
                    i32.add
                    set_local 0
                    get_local 7
                    get_local 3
                    i32.store offset=28
                    get_local 7
                    i32.const 16
                    i32.add
                    tee_local 1
                    i32.const 0
                    i32.store offset=4
                    get_local 1
                    i32.const 0
                    i32.store
                    i32.const 1336
                    i32.load
                    tee_local 1
                    i32.const 1
                    get_local 3
                    i32.shl
                    tee_local 2
                    i32.and
                    i32.eqz
                    if  ;; label = @9
                      i32.const 1336
                      get_local 1
                      get_local 2
                      i32.or
                      i32.store
                      get_local 0
                      get_local 7
                      i32.store
                      get_local 7
                      get_local 0
                      i32.store offset=24
                      get_local 7
                      get_local 7
                      i32.store offset=12
                      get_local 7
                      get_local 7
                      i32.store offset=8
                      br 2 (;@7;)
                    end
                    get_local 0
                    i32.load
                    set_local 0
                    i32.const 25
                    get_local 3
                    i32.const 1
                    i32.shr_u
                    i32.sub
                    set_local 1
                    get_local 4
                    get_local 3
                    i32.const 31
                    i32.eq
                    if i32  ;; label = @9
                      i32.const 0
                    else
                      get_local 1
                    end
                    i32.shl
                    set_local 3
                    block  ;; label = @9
                      block  ;; label = @10
                        block  ;; label = @11
                          loop  ;; label = @12
                            get_local 0
                            i32.load offset=4
                            i32.const -8
                            i32.and
                            get_local 4
                            i32.eq
                            br_if 2 (;@10;)
                            get_local 3
                            i32.const 1
                            i32.shl
                            set_local 1
                            get_local 0
                            i32.const 16
                            i32.add
                            get_local 3
                            i32.const 31
                            i32.shr_u
                            i32.const 2
                            i32.shl
                            i32.add
                            tee_local 3
                            i32.load
                            tee_local 2
                            i32.eqz
                            br_if 1 (;@11;)
                            get_local 1
                            set_local 3
                            get_local 2
                            set_local 0
                            br 0 (;@12;)
                          end
                          unreachable
                        end
                        get_local 3
                        i32.const 1348
                        i32.load
                        i32.lt_u
                        if  ;; label = @11
                          call 3
                        else
                          get_local 3
                          get_local 7
                          i32.store
                          get_local 7
                          get_local 0
                          i32.store offset=24
                          get_local 7
                          get_local 7
                          i32.store offset=12
                          get_local 7
                          get_local 7
                          i32.store offset=8
                          br 4 (;@7;)
                        end
                        br 1 (;@9;)
                      end
                      get_local 0
                      i32.const 8
                      i32.add
                      tee_local 1
                      i32.load
                      tee_local 3
                      i32.const 1348
                      i32.load
                      tee_local 2
                      i32.ge_u
                      get_local 0
                      get_local 2
                      i32.ge_u
                      i32.and
                      if  ;; label = @10
                        get_local 3
                        get_local 7
                        i32.store offset=12
                        get_local 1
                        get_local 7
                        i32.store
                        get_local 7
                        get_local 3
                        i32.store offset=8
                        get_local 7
                        get_local 0
                        i32.store offset=12
                        get_local 7
                        i32.const 0
                        i32.store offset=24
                      else
                        call 3
                      end
                    end
                  end
                end
                get_local 13
                set_global 6
                get_local 9
                i32.const 8
                i32.add
                return
              end
            end
            i32.const 1780
            set_local 2
            loop  ;; label = @5
              block  ;; label = @6
                get_local 2
                i32.load
                tee_local 4
                get_local 6
                i32.le_u
                if  ;; label = @7
                  get_local 4
                  get_local 2
                  i32.load offset=4
                  i32.add
                  tee_local 10
                  get_local 6
                  i32.gt_u
                  br_if 1 (;@6;)
                end
                get_local 2
                i32.load offset=8
                set_local 2
                br 1 (;@5;)
              end
            end
            i32.const 0
            get_local 10
            i32.const -47
            i32.add
            tee_local 2
            i32.const 8
            i32.add
            tee_local 4
            i32.sub
            i32.const 7
            i32.and
            set_local 5
            get_local 2
            get_local 4
            i32.const 7
            i32.and
            if i32  ;; label = @5
              get_local 5
            else
              i32.const 0
            end
            i32.add
            tee_local 2
            get_local 6
            i32.const 16
            i32.add
            tee_local 12
            i32.lt_u
            if i32  ;; label = @5
              get_local 6
              tee_local 2
            else
              get_local 2
            end
            i32.const 8
            i32.add
            set_local 8
            get_local 2
            i32.const 24
            i32.add
            set_local 4
            get_local 1
            i32.const -40
            i32.add
            set_local 11
            i32.const 0
            get_local 0
            i32.const 8
            i32.add
            tee_local 7
            i32.sub
            i32.const 7
            i32.and
            set_local 5
            i32.const 1356
            get_local 0
            get_local 7
            i32.const 7
            i32.and
            if i32  ;; label = @5
              get_local 5
            else
              i32.const 0
              tee_local 5
            end
            i32.add
            tee_local 7
            i32.store
            i32.const 1344
            get_local 11
            get_local 5
            i32.sub
            tee_local 5
            i32.store
            get_local 7
            get_local 5
            i32.const 1
            i32.or
            i32.store offset=4
            get_local 7
            get_local 5
            i32.add
            i32.const 40
            i32.store offset=4
            i32.const 1360
            i32.const 1820
            i32.load
            i32.store
            get_local 2
            i32.const 4
            i32.add
            tee_local 5
            i32.const 27
            i32.store
            get_local 8
            i32.const 1780
            i64.load align=4
            i64.store align=4
            get_local 8
            i32.const 1788
            i64.load align=4
            i64.store offset=8 align=4
            i32.const 1780
            get_local 0
            i32.store
            i32.const 1784
            get_local 1
            i32.store
            i32.const 1792
            i32.const 0
            i32.store
            i32.const 1788
            get_local 8
            i32.store
            get_local 4
            set_local 0
            loop  ;; label = @5
              get_local 0
              i32.const 4
              i32.add
              tee_local 1
              i32.const 7
              i32.store
              get_local 0
              i32.const 8
              i32.add
              get_local 10
              i32.lt_u
              if  ;; label = @6
                get_local 1
                set_local 0
                br 1 (;@5;)
              end
            end
            get_local 2
            get_local 6
            i32.ne
            if  ;; label = @5
              get_local 5
              get_local 5
              i32.load
              i32.const -2
              i32.and
              i32.store
              get_local 6
              get_local 2
              get_local 6
              i32.sub
              tee_local 5
              i32.const 1
              i32.or
              i32.store offset=4
              get_local 2
              get_local 5
              i32.store
              get_local 5
              i32.const 3
              i32.shr_u
              set_local 1
              get_local 5
              i32.const 256
              i32.lt_u
              if  ;; label = @6
                get_local 1
                i32.const 3
                i32.shl
                i32.const 1372
                i32.add
                set_local 0
                i32.const 1332
                i32.load
                tee_local 2
                i32.const 1
                get_local 1
                i32.shl
                tee_local 1
                i32.and
                if  ;; label = @7
                  get_local 0
                  i32.const 8
                  i32.add
                  tee_local 1
                  i32.load
                  tee_local 2
                  i32.const 1348
                  i32.load
                  i32.lt_u
                  if  ;; label = @8
                    call 3
                  else
                    get_local 1
                    set_local 21
                    get_local 2
                    set_local 9
                  end
                else
                  i32.const 1332
                  get_local 2
                  get_local 1
                  i32.or
                  i32.store
                  get_local 0
                  i32.const 8
                  i32.add
                  set_local 21
                  get_local 0
                  set_local 9
                end
                get_local 21
                get_local 6
                i32.store
                get_local 9
                get_local 6
                i32.store offset=12
                get_local 6
                get_local 9
                i32.store offset=8
                get_local 6
                get_local 0
                i32.store offset=12
                br 3 (;@3;)
              end
              get_local 5
              i32.const 8
              i32.shr_u
              tee_local 0
              if i32  ;; label = @6
                get_local 5
                i32.const 16777215
                i32.gt_u
                if i32  ;; label = @7
                  i32.const 31
                else
                  get_local 5
                  i32.const 14
                  get_local 0
                  get_local 0
                  i32.const 1048320
                  i32.add
                  i32.const 16
                  i32.shr_u
                  i32.const 8
                  i32.and
                  tee_local 0
                  i32.shl
                  tee_local 1
                  i32.const 520192
                  i32.add
                  i32.const 16
                  i32.shr_u
                  i32.const 4
                  i32.and
                  tee_local 2
                  get_local 0
                  i32.or
                  get_local 1
                  get_local 2
                  i32.shl
                  tee_local 0
                  i32.const 245760
                  i32.add
                  i32.const 16
                  i32.shr_u
                  i32.const 2
                  i32.and
                  tee_local 1
                  i32.or
                  i32.sub
                  get_local 0
                  get_local 1
                  i32.shl
                  i32.const 15
                  i32.shr_u
                  i32.add
                  tee_local 0
                  i32.const 7
                  i32.add
                  i32.shr_u
                  i32.const 1
                  i32.and
                  get_local 0
                  i32.const 1
                  i32.shl
                  i32.or
                end
              else
                i32.const 0
              end
              tee_local 1
              i32.const 2
              i32.shl
              i32.const 1636
              i32.add
              set_local 0
              get_local 6
              get_local 1
              i32.store offset=28
              get_local 6
              i32.const 0
              i32.store offset=20
              get_local 12
              i32.const 0
              i32.store
              i32.const 1336
              i32.load
              tee_local 2
              i32.const 1
              get_local 1
              i32.shl
              tee_local 4
              i32.and
              i32.eqz
              if  ;; label = @6
                i32.const 1336
                get_local 2
                get_local 4
                i32.or
                i32.store
                get_local 0
                get_local 6
                i32.store
                get_local 6
                get_local 0
                i32.store offset=24
                get_local 6
                get_local 6
                i32.store offset=12
                get_local 6
                get_local 6
                i32.store offset=8
                br 3 (;@3;)
              end
              get_local 0
              i32.load
              set_local 0
              i32.const 25
              get_local 1
              i32.const 1
              i32.shr_u
              i32.sub
              set_local 2
              get_local 5
              get_local 1
              i32.const 31
              i32.eq
              if i32  ;; label = @6
                i32.const 0
              else
                get_local 2
              end
              i32.shl
              set_local 1
              block  ;; label = @6
                block  ;; label = @7
                  block  ;; label = @8
                    loop  ;; label = @9
                      get_local 0
                      i32.load offset=4
                      i32.const -8
                      i32.and
                      get_local 5
                      i32.eq
                      br_if 2 (;@7;)
                      get_local 1
                      i32.const 1
                      i32.shl
                      set_local 2
                      get_local 0
                      i32.const 16
                      i32.add
                      get_local 1
                      i32.const 31
                      i32.shr_u
                      i32.const 2
                      i32.shl
                      i32.add
                      tee_local 1
                      i32.load
                      tee_local 4
                      i32.eqz
                      br_if 1 (;@8;)
                      get_local 2
                      set_local 1
                      get_local 4
                      set_local 0
                      br 0 (;@9;)
                    end
                    unreachable
                  end
                  get_local 1
                  i32.const 1348
                  i32.load
                  i32.lt_u
                  if  ;; label = @8
                    call 3
                  else
                    get_local 1
                    get_local 6
                    i32.store
                    get_local 6
                    get_local 0
                    i32.store offset=24
                    get_local 6
                    get_local 6
                    i32.store offset=12
                    get_local 6
                    get_local 6
                    i32.store offset=8
                    br 5 (;@3;)
                  end
                  br 1 (;@6;)
                end
                get_local 0
                i32.const 8
                i32.add
                tee_local 2
                i32.load
                tee_local 1
                i32.const 1348
                i32.load
                tee_local 4
                i32.ge_u
                get_local 0
                get_local 4
                i32.ge_u
                i32.and
                if  ;; label = @7
                  get_local 1
                  get_local 6
                  i32.store offset=12
                  get_local 2
                  get_local 6
                  i32.store
                  get_local 6
                  get_local 1
                  i32.store offset=8
                  get_local 6
                  get_local 0
                  i32.store offset=12
                  get_local 6
                  i32.const 0
                  i32.store offset=24
                else
                  call 3
                end
              end
            end
          else
            i32.const 1348
            i32.load
            tee_local 2
            i32.eqz
            get_local 0
            get_local 2
            i32.lt_u
            i32.or
            if  ;; label = @5
              i32.const 1348
              get_local 0
              i32.store
            end
            i32.const 1780
            get_local 0
            i32.store
            i32.const 1784
            get_local 1
            i32.store
            i32.const 1792
            i32.const 0
            i32.store
            i32.const 1368
            i32.const 1804
            i32.load
            i32.store
            i32.const 1364
            i32.const -1
            i32.store
            i32.const 0
            set_local 2
            loop  ;; label = @5
              get_local 2
              i32.const 3
              i32.shl
              i32.const 1372
              i32.add
              tee_local 4
              get_local 4
              i32.store offset=12
              get_local 4
              get_local 4
              i32.store offset=8
              get_local 2
              i32.const 1
              i32.add
              tee_local 2
              i32.const 32
              i32.ne
              br_if 0 (;@5;)
            end
            get_local 1
            i32.const -40
            i32.add
            set_local 2
            i32.const 0
            get_local 0
            i32.const 8
            i32.add
            tee_local 4
            i32.sub
            i32.const 7
            i32.and
            set_local 1
            i32.const 1356
            get_local 0
            get_local 4
            i32.const 7
            i32.and
            if i32  ;; label = @5
              get_local 1
            else
              i32.const 0
              tee_local 1
            end
            i32.add
            tee_local 0
            i32.store
            i32.const 1344
            get_local 2
            get_local 1
            i32.sub
            tee_local 1
            i32.store
            get_local 0
            get_local 1
            i32.const 1
            i32.or
            i32.store offset=4
            get_local 0
            get_local 1
            i32.add
            i32.const 40
            i32.store offset=4
            i32.const 1360
            i32.const 1820
            i32.load
            i32.store
          end
        end
        i32.const 1344
        i32.load
        tee_local 0
        get_local 3
        i32.gt_u
        if  ;; label = @3
          i32.const 1344
          get_local 0
          get_local 3
          i32.sub
          tee_local 1
          i32.store
          i32.const 1356
          i32.const 1356
          i32.load
          tee_local 0
          get_local 3
          i32.add
          tee_local 2
          i32.store
          get_local 2
          get_local 1
          i32.const 1
          i32.or
          i32.store offset=4
          get_local 0
          get_local 3
          i32.const 3
          i32.or
          i32.store offset=4
          get_local 13
          set_global 6
          get_local 0
          i32.const 8
          i32.add
          return
        end
      end
      call 14
      i32.const 12
      i32.store
      get_local 13
      set_global 6
      i32.const 0
    end)
  (func (;18;) (type 2) (param i32)
    (local i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32 i32)
    block  ;; label = @1
      get_local 0
      i32.eqz
      if  ;; label = @2
        return
      end
      get_local 0
      i32.const -8
      i32.add
      tee_local 2
      i32.const 1348
      i32.load
      tee_local 12
      i32.lt_u
      if  ;; label = @2
        call 3
      end
      get_local 0
      i32.const -4
      i32.add
      i32.load
      tee_local 0
      i32.const 3
      i32.and
      tee_local 11
      i32.const 1
      i32.eq
      if  ;; label = @2
        call 3
      end
      get_local 2
      get_local 0
      i32.const -8
      i32.and
      tee_local 5
      i32.add
      set_local 7
      block  ;; label = @2
        get_local 0
        i32.const 1
        i32.and
        if  ;; label = @3
          get_local 2
          set_local 4
          get_local 2
          set_local 3
          get_local 5
          set_local 1
        else
          get_local 2
          i32.load
          set_local 9
          get_local 11
          i32.eqz
          if  ;; label = @4
            return
          end
          get_local 2
          i32.const 0
          get_local 9
          i32.sub
          i32.add
          tee_local 0
          get_local 12
          i32.lt_u
          if  ;; label = @4
            call 3
          end
          get_local 9
          get_local 5
          i32.add
          set_local 2
          get_local 0
          i32.const 1352
          i32.load
          i32.eq
          if  ;; label = @4
            get_local 7
            i32.const 4
            i32.add
            tee_local 1
            i32.load
            tee_local 3
            i32.const 3
            i32.and
            i32.const 3
            i32.ne
            if  ;; label = @5
              get_local 0
              set_local 4
              get_local 0
              set_local 3
              get_local 2
              set_local 1
              br 3 (;@2;)
            end
            i32.const 1340
            get_local 2
            i32.store
            get_local 1
            get_local 3
            i32.const -2
            i32.and
            i32.store
            get_local 0
            get_local 2
            i32.const 1
            i32.or
            i32.store offset=4
            get_local 0
            get_local 2
            i32.add
            get_local 2
            i32.store
            return
          end
          get_local 9
          i32.const 3
          i32.shr_u
          set_local 5
          get_local 9
          i32.const 256
          i32.lt_u
          if  ;; label = @4
            get_local 0
            i32.load offset=12
            set_local 3
            get_local 0
            i32.load offset=8
            tee_local 4
            get_local 5
            i32.const 3
            i32.shl
            i32.const 1372
            i32.add
            tee_local 1
            i32.ne
            if  ;; label = @5
              get_local 4
              get_local 12
              i32.lt_u
              if  ;; label = @6
                call 3
              end
              get_local 4
              i32.load offset=12
              get_local 0
              i32.ne
              if  ;; label = @6
                call 3
              end
            end
            get_local 3
            get_local 4
            i32.eq
            if  ;; label = @5
              i32.const 1332
              i32.const 1332
              i32.load
              i32.const 1
              get_local 5
              i32.shl
              i32.const -1
              i32.xor
              i32.and
              i32.store
              get_local 0
              set_local 4
              get_local 0
              set_local 3
              get_local 2
              set_local 1
              br 3 (;@2;)
            end
            get_local 3
            get_local 1
            i32.eq
            if  ;; label = @5
              get_local 3
              i32.const 8
              i32.add
              set_local 6
            else
              get_local 3
              get_local 12
              i32.lt_u
              if  ;; label = @6
                call 3
              end
              get_local 3
              i32.const 8
              i32.add
              tee_local 1
              i32.load
              get_local 0
              i32.eq
              if  ;; label = @6
                get_local 1
                set_local 6
              else
                call 3
              end
            end
            get_local 4
            get_local 3
            i32.store offset=12
            get_local 6
            get_local 4
            i32.store
            get_local 0
            set_local 4
            get_local 0
            set_local 3
            get_local 2
            set_local 1
            br 2 (;@2;)
          end
          get_local 0
          i32.load offset=24
          set_local 13
          block  ;; label = @4
            get_local 0
            i32.load offset=12
            tee_local 5
            get_local 0
            i32.eq
            if  ;; label = @5
              get_local 0
              i32.const 16
              i32.add
              tee_local 6
              i32.const 4
              i32.add
              tee_local 9
              i32.load
              tee_local 5
              if  ;; label = @6
                get_local 9
                set_local 6
              else
                get_local 6
                i32.load
                tee_local 5
                i32.eqz
                if  ;; label = @7
                  i32.const 0
                  set_local 8
                  br 3 (;@4;)
                end
              end
              loop  ;; label = @6
                get_local 5
                i32.const 20
                i32.add
                tee_local 9
                i32.load
                tee_local 11
                if  ;; label = @7
                  get_local 11
                  set_local 5
                  get_local 9
                  set_local 6
                  br 1 (;@6;)
                end
                get_local 5
                i32.const 16
                i32.add
                tee_local 9
                i32.load
                tee_local 11
                if  ;; label = @7
                  get_local 11
                  set_local 5
                  get_local 9
                  set_local 6
                  br 1 (;@6;)
                end
              end
              get_local 6
              get_local 12
              i32.lt_u
              if  ;; label = @6
                call 3
              else
                get_local 6
                i32.const 0
                i32.store
                get_local 5
                set_local 8
              end
            else
              get_local 0
              i32.load offset=8
              tee_local 6
              get_local 12
              i32.lt_u
              if  ;; label = @6
                call 3
              end
              get_local 6
              i32.const 12
              i32.add
              tee_local 9
              i32.load
              get_local 0
              i32.ne
              if  ;; label = @6
                call 3
              end
              get_local 5
              i32.const 8
              i32.add
              tee_local 11
              i32.load
              get_local 0
              i32.eq
              if  ;; label = @6
                get_local 9
                get_local 5
                i32.store
                get_local 11
                get_local 6
                i32.store
                get_local 5
                set_local 8
              else
                call 3
              end
            end
          end
          get_local 13
          if  ;; label = @4
            get_local 0
            get_local 0
            i32.load offset=28
            tee_local 5
            i32.const 2
            i32.shl
            i32.const 1636
            i32.add
            tee_local 6
            i32.load
            i32.eq
            if  ;; label = @5
              get_local 6
              get_local 8
              i32.store
              get_local 8
              i32.eqz
              if  ;; label = @6
                i32.const 1336
                i32.const 1336
                i32.load
                i32.const 1
                get_local 5
                i32.shl
                i32.const -1
                i32.xor
                i32.and
                i32.store
                get_local 0
                set_local 4
                get_local 0
                set_local 3
                get_local 2
                set_local 1
                br 4 (;@2;)
              end
            else
              get_local 13
              i32.const 1348
              i32.load
              i32.lt_u
              if  ;; label = @6
                call 3
              else
                get_local 13
                i32.const 16
                i32.add
                get_local 13
                i32.load offset=16
                get_local 0
                i32.ne
                i32.const 2
                i32.shl
                i32.add
                get_local 8
                i32.store
                get_local 8
                i32.eqz
                if  ;; label = @7
                  get_local 0
                  set_local 4
                  get_local 0
                  set_local 3
                  get_local 2
                  set_local 1
                  br 5 (;@2;)
                end
              end
            end
            get_local 8
            i32.const 1348
            i32.load
            tee_local 6
            i32.lt_u
            if  ;; label = @5
              call 3
            end
            get_local 8
            get_local 13
            i32.store offset=24
            get_local 0
            i32.const 16
            i32.add
            tee_local 9
            i32.load
            tee_local 5
            if  ;; label = @5
              get_local 5
              get_local 6
              i32.lt_u
              if  ;; label = @6
                call 3
              else
                get_local 8
                get_local 5
                i32.store offset=16
                get_local 5
                get_local 8
                i32.store offset=24
              end
            end
            get_local 9
            i32.load offset=4
            tee_local 5
            if  ;; label = @5
              get_local 5
              i32.const 1348
              i32.load
              i32.lt_u
              if  ;; label = @6
                call 3
              else
                get_local 8
                get_local 5
                i32.store offset=20
                get_local 5
                get_local 8
                i32.store offset=24
                get_local 0
                set_local 4
                get_local 0
                set_local 3
                get_local 2
                set_local 1
              end
            else
              get_local 0
              set_local 4
              get_local 0
              set_local 3
              get_local 2
              set_local 1
            end
          else
            get_local 0
            set_local 4
            get_local 0
            set_local 3
            get_local 2
            set_local 1
          end
        end
      end
      get_local 4
      get_local 7
      i32.ge_u
      if  ;; label = @2
        call 3
      end
      get_local 7
      i32.const 4
      i32.add
      tee_local 2
      i32.load
      tee_local 0
      i32.const 1
      i32.and
      i32.eqz
      if  ;; label = @2
        call 3
      end
      get_local 0
      i32.const 2
      i32.and
      if  ;; label = @2
        get_local 2
        get_local 0
        i32.const -2
        i32.and
        i32.store
        get_local 3
        get_local 1
        i32.const 1
        i32.or
        i32.store offset=4
        get_local 4
        get_local 1
        i32.add
        get_local 1
        i32.store
      else
        i32.const 1352
        i32.load
        set_local 2
        get_local 7
        i32.const 1356
        i32.load
        i32.eq
        if  ;; label = @3
          i32.const 1344
          i32.const 1344
          i32.load
          get_local 1
          i32.add
          tee_local 0
          i32.store
          i32.const 1356
          get_local 3
          i32.store
          get_local 3
          get_local 0
          i32.const 1
          i32.or
          i32.store offset=4
          get_local 3
          get_local 2
          i32.ne
          if  ;; label = @4
            return
          end
          i32.const 1352
          i32.const 0
          i32.store
          i32.const 1340
          i32.const 0
          i32.store
          return
        end
        get_local 7
        get_local 2
        i32.eq
        if  ;; label = @3
          i32.const 1340
          i32.const 1340
          i32.load
          get_local 1
          i32.add
          tee_local 0
          i32.store
          i32.const 1352
          get_local 4
          i32.store
          get_local 3
          get_local 0
          i32.const 1
          i32.or
          i32.store offset=4
          get_local 4
          get_local 0
          i32.add
          get_local 0
          i32.store
          return
        end
        get_local 0
        i32.const -8
        i32.and
        get_local 1
        i32.add
        set_local 6
        get_local 0
        i32.const 3
        i32.shr_u
        set_local 5
        block  ;; label = @3
          get_local 0
          i32.const 256
          i32.lt_u
          if  ;; label = @4
            get_local 7
            i32.load offset=12
            set_local 1
            get_local 7
            i32.load offset=8
            tee_local 2
            get_local 5
            i32.const 3
            i32.shl
            i32.const 1372
            i32.add
            tee_local 0
            i32.ne
            if  ;; label = @5
              get_local 2
              i32.const 1348
              i32.load
              i32.lt_u
              if  ;; label = @6
                call 3
              end
              get_local 2
              i32.load offset=12
              get_local 7
              i32.ne
              if  ;; label = @6
                call 3
              end
            end
            get_local 1
            get_local 2
            i32.eq
            if  ;; label = @5
              i32.const 1332
              i32.const 1332
              i32.load
              i32.const 1
              get_local 5
              i32.shl
              i32.const -1
              i32.xor
              i32.and
              i32.store
              br 2 (;@3;)
            end
            get_local 1
            get_local 0
            i32.eq
            if  ;; label = @5
              get_local 1
              i32.const 8
              i32.add
              set_local 15
            else
              get_local 1
              i32.const 1348
              i32.load
              i32.lt_u
              if  ;; label = @6
                call 3
              end
              get_local 1
              i32.const 8
              i32.add
              tee_local 0
              i32.load
              get_local 7
              i32.eq
              if  ;; label = @6
                get_local 0
                set_local 15
              else
                call 3
              end
            end
            get_local 2
            get_local 1
            i32.store offset=12
            get_local 15
            get_local 2
            i32.store
          else
            get_local 7
            i32.load offset=24
            set_local 8
            block  ;; label = @5
              get_local 7
              i32.load offset=12
              tee_local 0
              get_local 7
              i32.eq
              if  ;; label = @6
                get_local 7
                i32.const 16
                i32.add
                tee_local 1
                i32.const 4
                i32.add
                tee_local 2
                i32.load
                tee_local 0
                if  ;; label = @7
                  get_local 2
                  set_local 1
                else
                  get_local 1
                  i32.load
                  tee_local 0
                  i32.eqz
                  if  ;; label = @8
                    i32.const 0
                    set_local 10
                    br 3 (;@5;)
                  end
                end
                loop  ;; label = @7
                  get_local 0
                  i32.const 20
                  i32.add
                  tee_local 2
                  i32.load
                  tee_local 5
                  if  ;; label = @8
                    get_local 5
                    set_local 0
                    get_local 2
                    set_local 1
                    br 1 (;@7;)
                  end
                  get_local 0
                  i32.const 16
                  i32.add
                  tee_local 2
                  i32.load
                  tee_local 5
                  if  ;; label = @8
                    get_local 5
                    set_local 0
                    get_local 2
                    set_local 1
                    br 1 (;@7;)
                  end
                end
                get_local 1
                i32.const 1348
                i32.load
                i32.lt_u
                if  ;; label = @7
                  call 3
                else
                  get_local 1
                  i32.const 0
                  i32.store
                  get_local 0
                  set_local 10
                end
              else
                get_local 7
                i32.load offset=8
                tee_local 1
                i32.const 1348
                i32.load
                i32.lt_u
                if  ;; label = @7
                  call 3
                end
                get_local 1
                i32.const 12
                i32.add
                tee_local 2
                i32.load
                get_local 7
                i32.ne
                if  ;; label = @7
                  call 3
                end
                get_local 0
                i32.const 8
                i32.add
                tee_local 5
                i32.load
                get_local 7
                i32.eq
                if  ;; label = @7
                  get_local 2
                  get_local 0
                  i32.store
                  get_local 5
                  get_local 1
                  i32.store
                  get_local 0
                  set_local 10
                else
                  call 3
                end
              end
            end
            get_local 8
            if  ;; label = @5
              get_local 7
              get_local 7
              i32.load offset=28
              tee_local 0
              i32.const 2
              i32.shl
              i32.const 1636
              i32.add
              tee_local 1
              i32.load
              i32.eq
              if  ;; label = @6
                get_local 1
                get_local 10
                i32.store
                get_local 10
                i32.eqz
                if  ;; label = @7
                  i32.const 1336
                  i32.const 1336
                  i32.load
                  i32.const 1
                  get_local 0
                  i32.shl
                  i32.const -1
                  i32.xor
                  i32.and
                  i32.store
                  br 4 (;@3;)
                end
              else
                get_local 8
                i32.const 1348
                i32.load
                i32.lt_u
                if  ;; label = @7
                  call 3
                else
                  get_local 8
                  i32.const 16
                  i32.add
                  get_local 8
                  i32.load offset=16
                  get_local 7
                  i32.ne
                  i32.const 2
                  i32.shl
                  i32.add
                  get_local 10
                  i32.store
                  get_local 10
                  i32.eqz
                  br_if 4 (;@3;)
                end
              end
              get_local 10
              i32.const 1348
              i32.load
              tee_local 1
              i32.lt_u
              if  ;; label = @6
                call 3
              end
              get_local 10
              get_local 8
              i32.store offset=24
              get_local 7
              i32.const 16
              i32.add
              tee_local 2
              i32.load
              tee_local 0
              if  ;; label = @6
                get_local 0
                get_local 1
                i32.lt_u
                if  ;; label = @7
                  call 3
                else
                  get_local 10
                  get_local 0
                  i32.store offset=16
                  get_local 0
                  get_local 10
                  i32.store offset=24
                end
              end
              get_local 2
              i32.load offset=4
              tee_local 0
              if  ;; label = @6
                get_local 0
                i32.const 1348
                i32.load
                i32.lt_u
                if  ;; label = @7
                  call 3
                else
                  get_local 10
                  get_local 0
                  i32.store offset=20
                  get_local 0
                  get_local 10
                  i32.store offset=24
                end
              end
            end
          end
        end
        get_local 3
        get_local 6
        i32.const 1
        i32.or
        i32.store offset=4
        get_local 4
        get_local 6
        i32.add
        get_local 6
        i32.store
        get_local 3
        i32.const 1352
        i32.load
        i32.eq
        if  ;; label = @3
          i32.const 1340
          get_local 6
          i32.store
          return
        else
          get_local 6
          set_local 1
        end
      end
      get_local 1
      i32.const 3
      i32.shr_u
      set_local 4
      get_local 1
      i32.const 256
      i32.lt_u
      if  ;; label = @2
        get_local 4
        i32.const 3
        i32.shl
        i32.const 1372
        i32.add
        set_local 0
        i32.const 1332
        i32.load
        tee_local 1
        i32.const 1
        get_local 4
        i32.shl
        tee_local 4
        i32.and
        if  ;; label = @3
          get_local 0
          i32.const 8
          i32.add
          tee_local 1
          i32.load
          tee_local 4
          i32.const 1348
          i32.load
          i32.lt_u
          if  ;; label = @4
            call 3
          else
            get_local 1
            set_local 16
            get_local 4
            set_local 14
          end
        else
          i32.const 1332
          get_local 1
          get_local 4
          i32.or
          i32.store
          get_local 0
          i32.const 8
          i32.add
          set_local 16
          get_local 0
          set_local 14
        end
        get_local 16
        get_local 3
        i32.store
        get_local 14
        get_local 3
        i32.store offset=12
        get_local 3
        get_local 14
        i32.store offset=8
        get_local 3
        get_local 0
        i32.store offset=12
        return
      end
      get_local 1
      i32.const 8
      i32.shr_u
      tee_local 0
      if i32  ;; label = @2
        get_local 1
        i32.const 16777215
        i32.gt_u
        if i32  ;; label = @3
          i32.const 31
        else
          get_local 1
          i32.const 14
          get_local 0
          get_local 0
          i32.const 1048320
          i32.add
          i32.const 16
          i32.shr_u
          i32.const 8
          i32.and
          tee_local 0
          i32.shl
          tee_local 4
          i32.const 520192
          i32.add
          i32.const 16
          i32.shr_u
          i32.const 4
          i32.and
          tee_local 2
          get_local 0
          i32.or
          get_local 4
          get_local 2
          i32.shl
          tee_local 0
          i32.const 245760
          i32.add
          i32.const 16
          i32.shr_u
          i32.const 2
          i32.and
          tee_local 4
          i32.or
          i32.sub
          get_local 0
          get_local 4
          i32.shl
          i32.const 15
          i32.shr_u
          i32.add
          tee_local 0
          i32.const 7
          i32.add
          i32.shr_u
          i32.const 1
          i32.and
          get_local 0
          i32.const 1
          i32.shl
          i32.or
        end
      else
        i32.const 0
      end
      tee_local 4
      i32.const 2
      i32.shl
      i32.const 1636
      i32.add
      set_local 0
      get_local 3
      get_local 4
      i32.store offset=28
      get_local 3
      i32.const 0
      i32.store offset=20
      get_local 3
      i32.const 0
      i32.store offset=16
      block  ;; label = @2
        i32.const 1336
        i32.load
        tee_local 2
        i32.const 1
        get_local 4
        i32.shl
        tee_local 5
        i32.and
        if  ;; label = @3
          get_local 0
          i32.load
          set_local 0
          i32.const 25
          get_local 4
          i32.const 1
          i32.shr_u
          i32.sub
          set_local 2
          get_local 1
          get_local 4
          i32.const 31
          i32.eq
          if i32  ;; label = @4
            i32.const 0
          else
            get_local 2
          end
          i32.shl
          set_local 4
          block  ;; label = @4
            block  ;; label = @5
              block  ;; label = @6
                loop  ;; label = @7
                  get_local 0
                  i32.load offset=4
                  i32.const -8
                  i32.and
                  get_local 1
                  i32.eq
                  br_if 2 (;@5;)
                  get_local 4
                  i32.const 1
                  i32.shl
                  set_local 2
                  get_local 0
                  i32.const 16
                  i32.add
                  get_local 4
                  i32.const 31
                  i32.shr_u
                  i32.const 2
                  i32.shl
                  i32.add
                  tee_local 4
                  i32.load
                  tee_local 5
                  i32.eqz
                  br_if 1 (;@6;)
                  get_local 2
                  set_local 4
                  get_local 5
                  set_local 0
                  br 0 (;@7;)
                end
                unreachable
              end
              get_local 4
              i32.const 1348
              i32.load
              i32.lt_u
              if  ;; label = @6
                call 3
              else
                get_local 4
                get_local 3
                i32.store
                get_local 3
                get_local 0
                i32.store offset=24
                get_local 3
                get_local 3
                i32.store offset=12
                get_local 3
                get_local 3
                i32.store offset=8
                br 4 (;@2;)
              end
              br 1 (;@4;)
            end
            get_local 0
            i32.const 8
            i32.add
            tee_local 4
            i32.load
            tee_local 1
            i32.const 1348
            i32.load
            tee_local 2
            i32.ge_u
            get_local 0
            get_local 2
            i32.ge_u
            i32.and
            if  ;; label = @5
              get_local 1
              get_local 3
              i32.store offset=12
              get_local 4
              get_local 3
              i32.store
              get_local 3
              get_local 1
              i32.store offset=8
              get_local 3
              get_local 0
              i32.store offset=12
              get_local 3
              i32.const 0
              i32.store offset=24
            else
              call 3
            end
          end
        else
          i32.const 1336
          get_local 2
          get_local 5
          i32.or
          i32.store
          get_local 0
          get_local 3
          i32.store
          get_local 3
          get_local 0
          i32.store offset=24
          get_local 3
          get_local 3
          i32.store offset=12
          get_local 3
          get_local 3
          i32.store offset=8
        end
      end
      i32.const 1364
      i32.const 1364
      i32.load
      i32.const -1
      i32.add
      tee_local 0
      i32.store
      get_local 0
      if  ;; label = @2
        return
      else
        i32.const 1788
        set_local 0
      end
      loop  ;; label = @2
        get_local 0
        i32.load
        tee_local 1
        i32.const 8
        i32.add
        set_local 0
        get_local 1
        br_if 0 (;@2;)
      end
      i32.const 1364
      i32.const -1
      i32.store
    end)
  (func (;19;) (type 1)
    nop)
  (func (;20;) (type 3) (param i32) (result i32)
    (local i32 i32)
    block i32  ;; label = @1
      get_global 5
      i32.load
      tee_local 2
      get_local 0
      i32.const 15
      i32.add
      i32.const -16
      i32.and
      tee_local 0
      i32.add
      set_local 1
      get_local 0
      i32.const 0
      i32.gt_s
      get_local 1
      get_local 2
      i32.lt_s
      i32.and
      get_local 1
      i32.const 0
      i32.lt_s
      i32.or
      if  ;; label = @2
        call 2
        drop
        i32.const 12
        call 4
        i32.const -1
        return
      end
      get_global 5
      get_local 1
      i32.store
      get_local 1
      call 1
      i32.gt_s
      if  ;; label = @2
        call 0
        i32.eqz
        if  ;; label = @3
          i32.const 12
          call 4
          get_global 5
          get_local 2
          i32.store
          i32.const -1
          return
        end
      end
      get_local 2
    end)
  (global (;5;) (mut i32) (get_global 0))
  (global (;6;) (mut i32) (get_global 1))
  (global (;7;) (mut i32) (get_global 2))
  (global (;8;) (mut i32) (i32.const 0))
  (global (;9;) (mut i32) (i32.const 0))
  (global (;10;) (mut i32) (i32.const 0))
  (export "_malloc" (func 17))
  (export "_free" (func 18))
  (export "_emscripten_get_global_libc" (func 13))
  (export "_hello_world" (func 12))
  (export "_sbrk" (func 20))
  (export "runPostSets" (func 19))
  (export "stackAlloc" (func 5))
  (export "stackSave" (func 6))
  (export "stackRestore" (func 7))
  (export "establishStackSpace" (func 8))
  (export "setTempRet0" (func 10))
  (export "getTempRet0" (func 11))
  (export "setThrew" (func 9))
  (data (i32.const 1212) "\1c\05"))
