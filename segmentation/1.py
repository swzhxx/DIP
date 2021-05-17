# 要添加一个新单元，输入 '# %%'
# 要添加一个新的标记单元，输入 '# %% [markdown]'
# %%
import matplotlib.pyplot as plt
import numpy as np 
from scipy import ndimage as ndim 
from  skimage import filters, feature


# %%
f = plt.imread("./images/airport.tif")
plt.imshow(f , "gray")


# %%
# Gauss Blur 
f = f.copy()
g = ndim.gaussian_filter(f , sigma =3)

plt.imshow(g,"gray")


# %%
# 计算 x,y方向上的梯度
def gradient(f):
    
    gradient_x_kernel = np.array ([
    [
        -1 , -2 , -1
    ],[
        0 , 0 , 0
    ],[
       1, 2, 1
    ]
    ])
    gx = ndim.convolve(f,gradient_x_kernel)

    gradient_y_kernel = np.array([     
    [
        -1, 0, 1
    ],[
        -2,0,2
    ],[
        -1,0,1
    ]
    ])

    gy =  ndim.convolve(f,gradient_y_kernel)
    
    
    return gx ,gy


# %%
gx, gy = gradient(g)
plt.imshow(gx , 'gray')


# %%
plt.imshow(gy , "gray")


# %%
# 幅度图
def magnitude(gx , gy):   
    m = np.sqrt(gx ** 2 + gy ** 2)
    return m 
m = magnitude(gx , gy)

plt.imshow(m , "gray")


# %%

"""
tan 算出梯度方向
"""
def direction (gx,gy):    
    return np.arctan2(gx,gy)

# 非极大化抑制    
def non_max_suppression(m , d):
    h,w = m.shape 
    res = np.zeros([h,w])    
    piece = np.pi / 8
    for y in range(1,h-1):
        for x in range(1,w-1):
            theta = d[y][x]
            current  = m[y][x]
            before = 0 
            after = 0             
            if 0 <= theta < piece or 6 * piece <= theta < np.pi:
                before = m[y][x -1 ]
                after = m[y][x + 1]
            if piece <= theta < piece * 3:
                before = m[ y + 1] [x - 1]
                after = m[y-1][x + 1]
            if piece * 3 <= theta < piece  * 5:
                before = m[y-1][x]
                after = m [y+1][x]
            if  5 * piece <= theta < 7 * piece:
                before = m[y-1][x-1]
                after = m[y+1][x+1]

            if current > after and current > before:
                 res[y][x] = current
            else : 
                res[y][x] = 0
    return res

d = direction(gx,  gy)

nm = non_max_suppression(m,d)
plt.imshow(nm , "gray")


# %%
## 双门限处理
def threshold(g , low_threshold_ratio = 0.5 , high_threshold_radio = 1 , weak = 25, strong=255):
    high_threshold = g.max() * high_threshold_radio
    low_threshold = high_threshold * low_threshold_ratio
    res = np.zeros(g.shape)  
    strong_y , strong_x = np.where(g > high_threshold)
    zeros_y , zeros_x = np.where(g < low_threshold)
    weak_y , weak_x = np.where((g <= high_threshold) & (g>=low_threshold))
    res[strong_y,strong_x] = strong
    res[weak_y , weak_x] = weak
    res[zeros_y , zeros_x] = 0
    return res 

res = threshold(nm , weak = 0 , high_threshold_radio = 0.2 , low_threshold_ratio = 0.1)
plt.imshow(res,"gray" )


# %%
"""
保存个图像 霍夫变换用
temp = threshold(nm , weak = 0 , high_threshold_radio =0.9)
plt.imshow(temp,"gray" )
plt.axis("off")
plt.imsave("airport_edge.png" , temp)
"""


# %%

edges = feature.canny(f,sigma = 3)

plt.imshow(edges , "gray")


# %%

sh = filters.sobel_h(f)

plt.imshow(sh , "gray")


# %%
sv = filters.sobel_v(f)
plt.imshow(sv,"gray")


# %%



