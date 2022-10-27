// pub enum ExpectKind {
//     KeywordToGetMeaning,
//     ExprAsCondition,
//     WordorNone, // 语句或块
//     ExprAsReturn,
//     BraToSetDef,
//     AssToSetVul,
//     ExpToSetStruct,
//     ExprAsArgs,
//     KeyWordAsFunReturnType,
//     ExprAsVul,
//     BlockForFun,
//     BlockForIf,
//     BlockForElse,
//     SBlockForStruct,
//     ExprAsWord,
//     //...
// }
pub enum Expectkind {
    KeywordForSet,
    KeywordForName,
    ExprAsVul,
    ExprAsArg,
    BlockForFIE, // FIE: fn if else
                 // SSlockForStruct,
} /*
  <W_IF $expr $block +$else>
  <W_ELSE $block>
  *<W_IMPL $name $sblock1> 改用匿名函数
  <W_DEF_FUN $name $args $type $block>
  <W_DEF_VUL $name $expr>
  <W_DEF_STRUCT $name $sblock2>
  */
