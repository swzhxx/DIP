# 要添加一个新单元，输入 '# %%'
# 要添加一个新的标记单元，输入 '# %% [markdown]'
# %% [markdown]
# # 尺度不变特征变换(Scale-invariant feature transform , SIFT)
# %% [markdown]
# SIFT算法的第一阶段是找到对尺度变化不变的图像位置。 这是在所有可能的尺度上使用一个称为尺度空间的函数搜索稳定的特征实现的，尺度空间是一种多尺度表示，它以一致的方式处理不同尺度的图像结构。基本思想是以某种形式来处理这样一个事实。由于事先并不知道这些尺度，因此一种合理的方法是同时处理所有相关的尺度。尺度空间将图像表示为平滑后的图像的一个参数族，目的时模拟图像的尺度减小时出现的细节损失。控制平滑的参数成为尺度参数。
# 
# 在SIFT算法中，高斯核用于实现平滑，因此尺度参数是标准差。使用高斯核的依据是Lindberg的研究成果，他证明只有满足一组重要约束条件的平滑核才是高斯低通核。因此，灰度图像$f(x,y)$的尺度空间
# $L(x,y,\sigma )$是$f$与一个可变尺度高斯核$G(x,y,\sigma)$的卷积：
# $$
#     L(x,y,\sigma) = G(x,y,\sigma) \star f(x,y)
# $$
# 
# 式中，尺度参数$\sigma$控制，G的形式如下：
# $$
#  G(x,y,\sigma) = \frac{1}{2\pi \sigma} e^{(-x^2 + y^2) / 2\sigma^2}
# $$
# 输入图像$f(x,y)$依次与标准差$\sigma , k\sigma , k^2\sigma,k^3\sigma$的高斯核卷积，生成一堆由常量因子$k$分割的高斯滤波图像。
# 
# SIFT 将尺度空间分为倍频程，每个倍频程对应与$\sigma$的加倍，类似与音乐理论中的一个倍频程对应于声音信号的频率倍增。SIFT将每个倍频程进一步细分为整数s个区间，因此区间1由两幅图像组成，区间2由三幅图像组成，以此类推。于是可以证明，在生成与一个倍频程对应的图像的高斯核中，所用的值是$k^s\sigma = 2\sigma$，这意味这$k = 2 ^ {1/s}$。 例如，对于 $s = 2 , k = \sqrt{2}$,并且连续地使用标准差$\sigma , \sqrt{2}\sigma , (\sqrt(\sigma))^2$来平滑图像，以便使用标准差为$\sqrt{2}^2\sigma = 2\sigma$的高斯核对序列中的第三副图像(即 s = 2 的倍频程)进行滤波。
# 
# 

# %%
import numpy as np 
import matplotlib.pyplot as plt 
import scipy.ndimage as ndimage
import cv2

# %%
f = plt.imread("./images/building-600by600.tif")
plt.imshow(f , "gray")


# %%

def create_dog(f , s = 2 , sigma = 1.6 , n_ocatives = None ):
    h,w = f.shape 
    if n_ocatives is None:
        n_ocatives = np.log2(min(h,w)) - 3
    k = pow(2 , 1/s)   
    gc = s + 2
    gs = []
    temp = []
    for cloth in range(gc):
        g =  ndimage.gaussian_filter(f , pow(k , cloth) * sigma)
        temp.append(g)
    gs.append(temp)
    if n_ocatives > 1 :
        gs.append(create_dog(cv2.resize(temp[-3],(int(h/2) , int(w/2))), s , sigma , n_ocatives = n_ocatives - 1 ))
    return gs 


# %%
dog = create_dog(f)




fig = plt.figure()

dog_l = len(dog)

for gs in dog:
    j = 0
    for g in gs:
        ax = fig.add_subplot(dog_l * 100 + dog_l * 10 + j +1)
        ax.imshow(g,"gray")
        j = j + 1
    
plt.show()
    



