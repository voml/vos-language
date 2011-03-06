# `vos-contract`

`vos-contract` is the VOS semantic consumer boundary for Oak-owned syntax
trees. It does not lex, parse, recover syntax, or define a second AST parser.
Callers obtain `oak-vos::VosRoot` from Oak and pass it to
`ContractInput::from_oak`.
