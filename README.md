# レポート課題：線形型言語

## 仕様

### 構文

```
 $VAR  := アルファベットとアラビア数字で表記される変数

 $EXPR := $LET | $IF | $SPLIT | $FREE | $QVAL | $APP | $VAR

 $LET  := let $VAR : $TYPE = $EXPR { $EXPR }
 $IF   := if $EXPR { $EXPR } else { $EXPR }
 $SPLIT := split $EXPR as $VAR, $VAR { $EXPR }
 $FREE := free $EXPR
 $APP  := ( $EXPR $EXPR )

 $QUAL := lin | un

 値
 $QVAL := $QUAL $VAL
 $VAL  := true | false | < $EXPR , $EXPR > | $FN
 $FN   := fn $VAR : $TYPE { $EXPR }

 型
 $TYPE := $QUAL $PRIM
 $PRIM := bool |
          ( $TYPE * $TYPE )
          ( $TYPE -> $TYPE )
```

### 型付け規則

スライドを参考

## 実行方法

```
$ cargo run examples/ex8.lin
```

初期状態では、ex8.linの型付けが可能。

## サンプルファイル

examples/ex*.linが、型付けに成功すべきファイルで、
examples/err*.linが、型付けに失敗すべきファイルとなる。
