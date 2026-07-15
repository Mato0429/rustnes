
# Loopyレジスタ

yyy nn YYYYY XXXXX

- yyy : fineY
- nn   : Nametable
- YYYYY: coarseY
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

PPUによる入出力はVisibleスキャンライン、PreRenderスキャンラインでのみ行われる。

- L000-239: Visible
- L240    : PostRender
- L241-260: VBlank
- L261    : PreRender

## 背景タイルフェッチ

背景タイルフェッチはC1-C256(32回)とC321-C336(2回)で繰り返される。

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

## スプライトパターンフェッチ

スプライトパターンフェッチはC257-C320(8回)で繰り返される。

### C257-258: Copy Sprite X

- TileID を Bus(NT)[0x2000 | (v & 0x0FFF)] からフェッチ(未使用)
- セカンダリOAMのスプライトのX座標を対応するスプライトラッチにロード

### C259-260: Copy Sprite Attribute

- Bus(NT)[0x2000 | (v & 0x0FFF)] をリード(無視)
- セカンダリOAMのスプライトの属性を対応するスプライトラッチにロード

### C261-262: Sprite Pattern lo fetch

8x8モード:
- dy を scanline - spriteY で求める。垂直フリップが有効な場合は 7 - dy を使用
- PTlo を Bus(PT)[SprPTBase | (TileID << 4) | dy] からフェッチ

8x16モード:
- dy を scanline - spriteY で求める。垂直フリップが有効な場合は 15 - dy を使用
- SprPTBase を TileID & 0x01から求める
- MaskedTileID を 7 < dy の場合 TileID | 0x01 それ以外は TileID & 0xFE で求める
- PTlo を Bus(PT)[SprPTBase | (MaskedTileID << 4) | dy] からフェッチ

### C263-264: Sprite Pattern hi fetch

8x8モード:
- PThi を Bus(PT)[SprPTBase | (TileID << 4) | dy + 8] からフェッチ
- 対応するスプライトシフトレジスタに必要なら水平フリップを適用して PTlo, PThi をロード

8x16モード:
- PThi を Bus(PT)[SprPTBase | (MaskedTileID << 4) | dy | 0x08] からフェッチ
- 対応するスプライトシフトレジスタに必要なら水平フリップを適用して PTlo, PThi をロード

## ダミーリード

これを正しく実装しないとMMC5などのマッパーが動作しない可能性がある。

### C337-338: Unused fetch

- TileID を Bus(NT)[0x2000 | (v & 0x0FFF)] からフェッチ

### C339-340: Ignored read

- Bus(NT)[0x2000 | (v & 0x0FFF)] をリード
- L261 C339 レンダリングがスプライト、背景ともに有効で奇数フレームの場合
    ダミーリード後にL000 C000にジャンプする。

# スキャンラインとイベント

## VBlankセット: VBlank(L241 only)

### C001: Set VBlank flag

- VBlankフラグをセット

## 水平リセット: Visible & PreRender

### C256: Next scanline

- v.fineYをインクリメント

### C257: Seek to left

- v に tの水平情報(coarseX, ネームテーブル下位ビット) をコピー

## VBlank取り下げ: PreRender

### C001: Clear flags

- VBlankフラグをクリア
- Sprite0Hitフラグをクリア
- SpriteOverflowフラグをクリア

## 垂直リセット: PreRender

### C280-C304 Seek to top

- 毎サイクルvにtの垂直情報(fineY, coarseY, ネームテーブル上位ビット)をコピー

# OAMとスプライト

PPUによるOAMとスプライト操作はVisibleスキャンラインでのみ行われる。

## Secondary OAMクリア

C001-064(32回)で繰り返される。

### C001-C002: Secondary FF Transfer

- セカンダリOAMの1バイトをFFで埋める。

## スプライト評価

スプライト評価はC065-256の間で繰り返される

### 奇数サイクル

- プライマリOAMポインタからデータを読み取る。

### 偶数サイクル

消化サイクル(プライマリOAMポインタが一巡した)の場合:
- 何もしない(厳密には無効なコピーが発生するが、これをハンドルするデバイスが存在しないため)

セカンダリOAMが一杯になった場合:
- 読み取ったデータをY座標としてスキャンライン範囲内か評価する。(スプライトサイズに注意)
    スキャンライン範囲内:
    - プライマリOAMポインタを4インクリメントする
    - スプライトオーバーフローフラグをセットする

    スキャンライン範囲外:
    - プライマリOAMポインタを5インクリメントする(実機バグ。)

その他:
- 読み取ったデータをY座標としてスキャンライン範囲内か評価する。(スプライトサイズに注意)
    スキャンライン範囲内:
    - プライマリOAMからセカンダリOAMにデータをコピーする
    - プライマリOAMポインタとセカンダリOAMポインタを1インクリメントする

    スキャンライン範囲外:
    - プライマリOAMポインタを4インクリメントする

# レンダリング

Visibleスキャンライン C1-C256の間で行われる。
レンダリング自体は入出力を行わずレジスタの値から色を選んで出力する。

## 描画するピクセルの選択

- fineXを元に背景シフトレジスタを PTlo, PThi, ATlo, AThi 取り出してシフトする。
- BgPixelIndex を (AThi << 3) | (ATlo << 2) | (PThi <<1) | PTlo で求める。
- 各スプライトレジスタで dx を cycle - spriteX で求める。
- 0 <= dx < 8 であった場合、対応するスプライトシフトレジスタすべてをシフトする。
- シフトしたデータを合成し、SprPixelIndex を (AThi << 3) | (ATlo << 2) | (PThi <<1) | PTlo で求める。
- SprPixelIndexから最もOAM内スプライト優先度が高く透明ピクセル(下位2ビットが0)でないものを描画用に選択する。
- SprPixelIndex を (AThi << 3) | (ATlo << 2) | (PThi <<1) | PTlo で求める。
- BgPixelIndex, SprPixelIndex, スプライト優先度を用いて描画するピクセルを決定する。
    ただし、対応するフラグ(Mask Bit1, 2, 3, 4)により、インデックスは0として扱う。

## Sprite0Hit について

セカンダリOAMスプライト0の描画中にBgPixelIndexとSprPixelIndexがともに0でなく、
スプライトのXが255Sprite0Hitフラグをセットする。
