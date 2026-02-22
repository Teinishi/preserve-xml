# preserve-xml

`preserve-xml` is a non-destructive XML mapping library built on top of `quick-xml`. It allows you to parse XML into typed Rust structures while transparently preserving unmapped attributes and unknown child elements for round-trip serialization.

`preserve-xml` は `quick-xml` をベースにした、非破壊的な XML マッピングライブラリです。XML を Rust の構造体にパースしつつ、定義していない属性や未知の子要素を透過的に保持し、再出力時に復元することを可能にします。

## Key Features / 主な特徴
- Non-Destructive (Stay-put): Any XML tags or attributes not defined in your structs are kept as raw bytes and restored upon serialization.
- Clean API: Use `Content<T>` to process only over the data you care about, ignoring the "noise" of unknown elements.
- Metadata Awareness: Every node can carry its original unmapped attributes via `WithMetadata<T>`.

- 非破壊的: 構造体で定義されていないタグや属性は生のバイト列として保持され、シリアライズ時にそのまま復元されます。
- クリーンな API: `Content<T>` を使用することで、関心のない要素・属性を無視して、関心のあるデータだけを処理できます。
- メタデータの保持: `WithMetadata<T>` を通じて、各ノードは元の定義外属性を保持し続けます。

## Usage / 使い方

