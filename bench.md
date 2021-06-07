##single thread single folder
```
samo@MEDION:~/Namizje/lac$ time ./target/debug/lac /home/samo/Namizje/urejeno/ks
Using builtin lossless audio checker 2.0.5

real    0m15,063s
user    0m15,004s
sys     0m0,056s
samo@MEDION:~/Namizje/lac$ time ./target/release/lac /home/samo/Namizje/urejeno/ks
Using builtin lossless audio checker 2.0.5

real    0m0,495s
user    0m0,460s
sys     0m0,032s
```
deleted Lac.log
```
samo@MEDION:~/Namizje/lac$ time ./target/release/lac /home/samo/Namizje/urejeno/ks
Using builtin lossless audio checker 2.0.5

real    0m16,252s
user    1m8,816s
sys     0m0,308s
samo@MEDION:~/Namizje/lac$ time ./target/debug/lac /home/samo/Namizje/urejeno/ks
Using builtin lossless audio checker 2.0.5

real    1m38,276s
user    2m32,617s
sys     0m0,540s
```