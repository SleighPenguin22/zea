Known Issues:

# THR ident-expressions of a global should first load the global, and then use it
QBE requires us to load/copy a global before using it.


The following is not allowed:
```
export function ub $main() {
@main
  %t0 =w add 5, $main_global_a
  %main_main_local_c =w copy %t0
  ret %main_main_local_c
  # END_main
}
export data $main_global_a = align 1 { w 67 }
```

Because it should actually be the following:
```
export function ub $main() {
@main
  %main_global_a =w copy $main_global_a
  %t0 =w add 5, %main_global_a
  %main_main_local_c =w copy %t0
  ret %main_main_local_c
  # END_main
}
export data $main_global_a = align 1 { w 67 }
	
```
