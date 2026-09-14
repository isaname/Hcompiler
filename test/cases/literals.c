// 十进制、八进制、十六进制字面量与一元运算
/* 块注释也应被跳过 */
int main() {
    int dec = 100;
    int oct = 0755;
    int hex = 0xFF;
    output(dec);
    output(oct);
    output(hex);
    output(-dec);
    output(+hex);
    output(!0);
    output(!dec);
    output(-(-oct));
    output(hex / dec + hex % dec);
    return 0;
}
