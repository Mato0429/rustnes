
# Loopyレジスタ

yyyy nn YYYYY XXXXX

- yyyy : fineY
- nn   : Nametable
- YYYYY: coraseY
- XXXXX: coarseX

## coarseXのインクリメント

31が最大で、それ以降は0にラップアラウンドする。
その際、自身のネームテーブル下位ビットを反転させる。

## fineYのインクリメント

7が最大で、それ以降は0にラップアラウンドする。
その際、coarseYをインクリメント(繰り上げ)する。

## coarseYのインクリメント

29が最大で、それ以降は0にラップアラウンドする。
その際、自身のネームテーブル上位ビットを反転させる。

# スキャンラインと入出力

- L000-239: Visible
- L240    : PostRender
- L241-260: VBlank
- L261    : PreRender

## 背景タイルフェッチ: Visible & PreRender

- 背景タイルフェッチはC1-C256(32回)とC321-C336(2回)で繰り返される。

### C001-002: Background Nametable fetch

- TileID を Bus(NT)[0x2000 | (v & 0x0FFF)] からフェッチ

### C003-004: Background Attribute fetch

- ATByte を Bus(AT)[0x23C0 | (v & 0x0C00) | ((v >> 4) & 0x38) | ((v >> 2) & 0x07)] からフェッチ
- ATlo を (ATByte >> (((v.coarseY & 0x2) << 1) | v.coarseX & 0x2)) & 0x1 で計算
- AThi を (ATByte >> (((v.coarseY & 0x2) << 1) | v.coarseX & 0x2)) & 0x2 で計算

### C005-006: Background Pattern lo fetch

- PTlo を Bus(PT)[BgPTBase | (TileID << 4) | v.fineY] からフェッチ

### C007-008: Background Pattern hi fetch

- PThi を Bus(PT)[BgPTBase | (TileID << 4) | v.fineY + 8] からフェッチ
- 背景シフトレジスタに PTlo, PThi, ATlo, AThi をロード
- v.coarseXをインクリメント

## スプライトパターンフェッチ: Visible & PreRender

- スプライトパターンフェッチはC257-C262(8回)で繰り返される。

### C257-258: Copy Sprite X

- TileID を Bus(NT)[0x2000 | (v & 0x0FFF)] からフェッチ(未使用)
- セカンダリOAMのスプライトのX座標を対応するスプライトラッチにロード

### C259-260: Copy Sprite Attribute

- Bus(NT)[0x2000 | (v & 0x0FFF)] をリード(無視)
- セカンダリOAMのスプライトの属性を対応するスプライトラッチにロード

### C261-262: Sprite Pattern lo fetch

8x8モード:
- dy を scanline - spriteY で求める。垂直フリップが有効な場合は 7 - dy を使用
- PTlo を Bus(PT)[BgPTBase | (TileID << 4) | dy] からフェッチ

8x16モード:
- 

### C263-264: Sprite Pattern hi fetch

8x8モード:
- PThi を Bus(PT)[BgPTBase | (TileID << 4) | dy + 8] からフェッチ
- 対応するスプライトシフトレジスタに必要なら水平フリップを適用して PTlo, PThi をロード

8x16モード:
- 

# スキャンラインとイベント

## VBlank Assertion: VBlank(L241 only)

### C001: Set VBlank flag

- VBlankフラグをセット

## Horizontal reset: Visible & PreRender

### C256: Next scanline

- v.fineYをインクリメント

### C257: Seek to left

- v に tの水平情報(coarseX, ネームテーブル下位ビット) をコピー

## Vertical reset: PreRender

### C280-C304 Seek to top

- 毎サイクルvにtの垂直情報(fineY, coarseY, ネームテーブル上位ビット)をコピー


L241     C001    : VBlankAssert : VBlank(StatBit6)をセット
