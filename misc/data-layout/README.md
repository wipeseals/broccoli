# Data Layout

## 誤り訂正

悩んだので ECC 内臓の上位モデルを参考に設計する

- w/o ECC (こちらを使用している) [KIOXIA_TC58NVG0S3HTA00](https://mm.digikey.com/Volume0/opasdata/d220001/medias/docus/506/KIOXIA_TC58NVG0S3HTA00_Rev2.00_E191001C.pdf)
- w/ ECC [KIOXIA_TC58BVG0S3HTA00](https://mm.digikey.com/Volume0/opasdata/d220001/medias/docus/506/KIOXIA_TC58BVG0S3HTA00_REV2.00_E20191001C.pdf)

### Datasheet 確認

#### ECC & Sector definition for ECC

- 2KByte page は 512byte \* 4area + 16byte \* 4area で使用している
- main + spare の 512+16=528byte ごとに、 9bit の検出と 8bit の訂正能力を備える
- ECC parity code は 2112-2175 (64byte) に格納するので、ここにはアクセスできないという注意書き

main+spare ごと ECC のペアになっている点、ECC Status Read の戻り値が Area 事に分かれていること、最後の ECC parity code 格納から以下のように推測できる（実際は不明）

| sector | main [byte] | spare [byte] | ECC parity [byte] |
| ------ | ----------- | ------------ | ----------------- |
| 1st    | 0-511       | 2048 - 2063  | 2112 - 2127       |
| 2nd    | 512 - 1023  | 2064 - 2079  | 2128 - 2143       |
| 3rd    | 1024 - 1535 | 2080 - 2095  | 2144 - 2159       |
| 4th    | 1536 - 2047 | 2096 - 2111  | 2160 - 2175       |

(17) Reliability Guidance によると 8bit ECC for each 512byte 必要 (これは w/o ECC model にも記載あり) とある。
(en Wikipedia より) additional parity ありの拡張 Hamming(256,247)だと考えると、data 部 247bit ごとに ECC Parity が 9bit 付与されることになる。

512+12byte 分付与すると 20byte 分の ECC parity code 格納が必要なので若干足りていないように見える。何か見落としがあるだろうか。

#### 0 padding 回避

同じ (17) Reliability Guidance で見つけたが 0 data padding をやめろという記載もある。この USB メモリには All Zero のバイナリを置かないでください、とは言えないので対策が必要そう。
だんだん CD の不揮発化フォーマットみたくなりつつあるような気もするが、データをアナログな物理現象に変換して格納し、何らかのエラーや物理都合の対策があるという点では同じなのかもしれない。

最初 128b130b などを考えたが、プロセッサに計算させるのに余り向いていない（用途考えれば自明だが）、再考する。誤り訂正符号の計算のように xor だけ、などで 32bit 単位で処理できると望ましい。
All 0 の領域を作らなければよいので、疑似乱数との xor でも良い気がしている。
偶然データが疑似乱数性生値と同一になることを懸念したが、一定の単位で疑似乱数を変更すれば確率的には下げられそうではある。
具体的にどの程度 0 データが続くことを避けなければいけない、などが不明瞭なのでるのでこれで妥協したい。

#### CRC

どう使うかはあまり考えていなかったが（実装中に欲しくなったときに使う事になりそう）、先の採択 pros について考えていて ECC の保護単位を細かくすることのデメリットを考えていなかった。
ずばり検査数が倍になるのである。ECC 分割数分だけ検査が必要なので Parity 計算と一致を確認しなければならない。

RP2040 は小規模であまり余裕のないプロセッサだと思うので、xor で実装できるといえど 2k data の読み出しのたびにこの計算をやるのはちょっと辛い気がする。
そもそも訂正が必要かどうかに気づくことが高速に行えればよいので、CRC などを付与するのは良いかと考えた。
幸い RP2040 は (おそらく USB IP で使用する目的だと思うが) DMA の Snoop で CRC 計算ができるので、これを付与するのが良さそうに思う。

これまた Encode 同様適用順序に悩むが、CRC はあくまで訂正の要否判断なので、最終的なデータで CRC 検査できることが望ましい。
また DMA で Snoop する都合上、Data buffer の一番最初か最後に付与しておいて、それ以外全部の領域をチェインなし一度の DMA Kick で済ませられるとよさそう。
DMA の DataUnitSize を 4byte にすることも鑑みると、4byte 単位で開けておく。

#### データ配置総括

ECC Unit の小数部は、area の byte 数に対して ECC 保護単位がアラインされておらず、ECC Parity code が複数領域にまたがった bit を用いて生成されていることを指す (1unit=247bit)

| description     | area [byte]     | ECC Unit | ecc: 拡張 Hamming(256, 247)                                                                                                                           |
| --------------- | --------------- | -------- | ----------------------------------------------------------------------------------------------------------------------------------------------------- |
| usb host data0  | 0 - 511         | ~16.58   | 512byte data                                                                                                                                          |
| usb host data1  | 512 - 1023      | ~33.16   | 512byte data                                                                                                                                          |
| usb host data2  | 1024 - 1535     | ~49.74   | 512byte data                                                                                                                                          |
| usb host data3  | 1536 - 2047     | ~66.33   | 512byte data                                                                                                                                          |
| spare data0     | 2048 - 2055     | ~66.59   | 8byte debug data                                                                                                                                      |
| spare data1     | 2056 - 2063     | ~66.85   | 8byte debug data                                                                                                                                      |
| spare data2     | 2064 - 2071     | ~67.11   | 8byte debug data                                                                                                                                      |
| spare data3     | 2072 - 2079     | ~67.36   | 8byte debug data                                                                                                                                      |
| meta data       | 2080 - 2091     | ~67.75   | 12byte debug data                                                                                                                                     |
| -               | (2092 - 2099.5) | ~68      | ECC 保護 68unit 目が 2068.625byte~2099.5byte を保護するが 2092byte 以後は Parity code 配置するため有効なデータは配置できず、ECC 計算時は 0 として扱う |
| ecc parity code | 2092 - 2171     | -        | `2092[byte]*8[bit/byte]/247[data bit/ecc unit]=68[ecc unit], align_u32(68[ecc unit]*9[bit/ecc unit]/8[bit/byte])=80[byte]`                            |
| crc code        | 2172 - 2175     | -        | 4byte crc (code に 4byte 使わないなら 0 埋め)                                                                                                         |
