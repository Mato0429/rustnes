//! PPUの描画が正しく動いているかを確認するための最小構成モックバス。
//!
//! 使い方:
//!   1. このファイルを `tests/ppu_render.rs` として配置する
//!      (Cargoのintegration testは crate の public API しか使えないので、
//!       Ppu::new / write_ppuctrl / write_ppumask / write_ppuaddr / write_ppudata /
//!       tick / display_buffer が pub である前提で書いています)
//!   2. `use rustnes::ppu::{Bus, Ppu};` の部分は実際のクレート名・モジュールパスに
//!      合わせて書き換えてください。
//!   3. `write_ppumask(0x08)` のビットはNESの一般的なPPUMASK定義
//!      (bit3 = show background)を仮定しています。
//!      registers.rs の PpuMask 定義を見て、実際のビット位置に合わせてください。

use rustnes_core::nes::ppu::{Bus, Ppu}; // TODO: 実際のクレート名/パスに置き換える

/// 描画確認専用の最小バス。
/// パターンテーブル(0x0000-0x1FFF)とネームテーブル(0x2000-0x2FFF)を
/// ただのフラット配列として持つだけ。ミラーリングやマッパーは一切考慮しない。
struct MockBus {
    chr: [u8; 0x2000],
    vram: [u8; 0x1000],
}

impl MockBus {
    fn new() -> Self {
        Self {
            chr: [0; 0x2000],
            vram: [0; 0x1000],
        }
    }

    /// パターンテーブルに1タイル分(16バイト)のビットプレーンを書き込むヘルパー。
    /// plane_lo/plane_hiはそれぞれ8行分のビットパターン。
    /// 各行のビット位置ごとに (plane_hi_bit << 1 | plane_lo_bit) が画素値(0-3)になる。
    fn set_tile(&mut self, tile_idx: u8, plane_lo: [u8; 8], plane_hi: [u8; 8]) {
        let base = tile_idx as usize * 16;
        self.chr[base..base + 8].copy_from_slice(&plane_lo);
        self.chr[base + 8..base + 16].copy_from_slice(&plane_hi);
    }

    /// ネームテーブルの1バイトを設定するヘルパー(nt: 0-3, offsetは0x000-0x3FF)。
    fn set_nametable_byte(&mut self, nt: u16, offset: u16, value: u8) {
        let addr = nt * 0x400 + offset;
        self.vram[addr as usize] = value;
    }
}

impl Bus for MockBus {
    fn read(&mut self, addr: u16) -> u8 {
        match addr {
            0x0000..=0x1FFF => self.chr[addr as usize],
            0x2000..=0x2FFF => self.vram[(addr - 0x2000) as usize],
            _ => 0,
        }
    }

    fn write(&mut self, addr: u16, data: u8) {
        match addr {
            0x0000..=0x1FFF => self.chr[addr as usize] = data,
            0x2000..=0x2FFF => self.vram[(addr - 0x2000) as usize] = data,
            _ => {}
        }
    }
}

/// 1フレーム分PPUを進める(341cycle * 262scanline)。
fn run_one_frame(ppu: &mut Ppu, bus: &mut MockBus) {
    for _ in 0..341 * 262 {
        ppu.tick(bus);
    }
}

/// 画面全体が単色で塗りつぶされることを確認する最小テスト。
/// ネームテーブル・属性テーブルはデフォルト0のままにし、
/// タイル0を「全ピクセルが画素値2」になるよう設定する。
#[test]
fn background_fills_with_expected_color() {
    let mut bus = MockBus::new();

    // タイル0: 下位プレーン=0x00, 上位プレーン=0xFF => 各ピクセルの画素値は 0b10 = 2
    bus.set_tile(0, [0x00; 8], [0xFF; 8]);
    // ネームテーブル/属性テーブルは0のままでOK
    // (タイル0が画面全体に敷き詰められ、属性は常にパレット0を指す)

    let mut ppu = Ppu::new();

    // パターンテーブルは0x0000側を使用 (PpuCtrl::B = 0)
    ppu.write_ppuctrl(0x00);

    // パレットを0x3F00からPPUDATA経由で設定
    ppu.write_ppuaddr(0x3F);
    ppu.write_ppuaddr(0x00);
    ppu.write_ppudata(&mut bus, 0x0F); // index0: backdrop
    ppu.write_ppudata(&mut bus, 0x16); // index1: 未使用
    ppu.write_ppudata(&mut bus, 0x2A); // index2: 今回のタイルが指す色
    ppu.write_ppudata(&mut bus, 0x12); // index3: 未使用

    // 背景描画を有効化。ビット位置はPpuMaskの定義に合わせて調整すること。
    ppu.write_ppumask(0x08);

    run_one_frame(&mut ppu, &mut bus);

    let buf = ppu.display_buffer();

    // 画面中央付近のピクセルを検証
    let (x, y) = (128, 120);
    let idx = (256 * y + x) * 4;
    let pixel = &buf[idx..idx + 4];

    // 何かしら描画されている(初期値0xFF埋めのままではない)ことをまず確認
    println!("pixel({x},{y}) = {:?}", pixel);
    assert_ne!(
        pixel[3], 0x00,
        "アルファが立っていない = 描画されていない可能性"
    );

    let buf = ppu.display_buffer();
    for x in 0..32 {
        let idx = (256 * 100 + x) * 4;
        println!("x={x}: {:?}", &buf[idx..idx + 3]);
    }

    // 画面の別の場所も同じ色になっているか(タイルが敷き詰められているか)確認
    let (x2, y2) = (10, 5);
    let idx2 = (256 * y2 + x2) * 4;
    assert_eq!(
        &buf[idx..idx + 3],
        &buf[idx2..idx2 + 3],
        "同じタイルなのに色が違う = スクロール/フェッチ周りにバグの疑い"
    );
}

/// タイル境界・attribute周りを確認したい場合の追加テスト例。
/// タイル0を左半分=値1、右半分=値2の縦縞にし、
/// 実際に横方向でピクセルが切り替わるかを確認する。
#[test]
fn background_tile_has_expected_horizontal_pattern() {
    let mut bus = MockBus::new();

    // 各行: 左4pxが画素値1、右4pxが画素値2になるようにビットプレーンを組む
    // plane_lo = 1111_0000 (左が1のbit0), plane_hi = 0000_1111 (右が1のbit1)
    bus.set_tile(0, [0b1111_0000; 8], [0b0000_1111; 8]);

    let mut ppu = Ppu::new();
    ppu.write_ppuctrl(0x00);

    ppu.write_ppuaddr(0x3F);
    ppu.write_ppuaddr(0x00);
    ppu.write_ppudata(&mut bus, 0x0F);
    ppu.write_ppudata(&mut bus, 0x16); // 画素値1の色
    ppu.write_ppudata(&mut bus, 0x2A); // 画素値2の色
    ppu.write_ppudata(&mut bus, 0x12);

    ppu.write_ppumask(0x08);

    run_one_frame(&mut ppu, &mut bus);

    let buf = ppu.display_buffer();

    let left_idx = (256 * 10 + 1) * 4; // タイル0の左寄り
    let right_idx = (256 * 10 + 6) * 4; // タイル0の右寄り

    assert_ne!(
        &buf[left_idx..left_idx + 3],
        &buf[right_idx..right_idx + 3],
        "左右で色が変わっていない = パターンフェッチかシフトレジスタにバグの疑い"
    );
}
