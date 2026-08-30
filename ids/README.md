# ID Registry

`index.json` is the baseline registry for numeric protocol/error/capability and
gameplay Tag identifiers. Numeric values are unique within a namespace and are
never reused; a retired value stays `Deprecated` so delayed messages and old
evidence remain diagnosable. New ids must be added through an ADR and a fixture
before code generation. GAS Tag hierarchy and counts consume the complete
`Tag` namespace; no second vocabulary store is authoritative.
