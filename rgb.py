#!/usr/bin/env python

# https://entropymine.com/imageworsener/srgbformula/
# http://www.color.org/chardata/rgb/srgb.xalter
# https://en.wikipedia.org/wiki/SRGB
_A = 0.055
_B = 12.92
_C = 0.4045
_D = 0.0031308
_P = 2.4

_A = 0.055
_B = 12.92
_C = 0.404482362771082
_D = 0.00313066844250063
_P = 2.4
def srgb_to_rgb(rgb):
    srgb = []
    for v in rgb:
        if v <= _C:
            v = v / _B
        else :
            v = ((v + _A) / (1.0 + _A))**_P
        srgb.append(v)
    return srgb

def rgb_to_srgb(srgb):
    rgb = []
    for v in srgb:
        if v <= _D :
            v = v * _B
        else :
            v = (1.0 + _A) * v**(1.0/_P) - _A
        rgb.append(v)
    return rgb

c = [242, 204, 153]
print(c)
c1 = [x / 255.0 for x in c]
print(c1)
c2 = rgb_to_srgb(c1)
print(c2)
c3 = [round(x * 255.0) for x in c2]
print(c3)
c4 = srgb_to_rgb(c2)
print(c4)
c5 = [round(x * 255.0) for x in c4]
print(c5)

def agg_linear_to_sRGB(x):
    if (x <= 0.0031308) :
        return (x * 12.92)
    else :
        return 1.055 * (x**(1 / 2.4)) - 0.055
def agg_sRGB_to_linear(x):
    if x <= 0.04045 :
        return x / 12.92
    else :
        return ((x + 0.055) / (1.055))**2.4

#print(round(agg_linear_to_sRGB(153./255.) * 255.))
#print(round(agg_sRGB_to_linear(203./255.) * 255.))
print(srgb_to_rgb([203./255.])[0]*255.)
for i in range(153,154):
    v = srgb_to_rgb(rgb_to_srgb([i/255.]))[0]*255.
    print(i, v, i-v)
