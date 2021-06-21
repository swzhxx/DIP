# 颜色空间

## 彩色模型

今天我们使用的大多数彩色模型要么是面向硬件的，要么是面向应用的。就数字图像处理而言，实际中最常用的面向硬件的模型有：针对彩色显示器和彩色摄像机开发的**RGB**模型，针对彩色打印的**CMY**(青色，深红色，黄色)模型和**CMYK**（青色，深红色，黑色）模型;针对人们描述和解释颜色的方式开发的**HSI**（色调，饱和度，亮度）模型。HSI 模型还有一个优点，即它能够接触图像中的颜色和灰度级信息的联系，使其更适合于本书中开发的灰度级处理技术。

## CMY，CMYK 彩色模型

$$
  \begin{bmatrix}
    C \\
    M \\
    Y \\
  \end{bmatrix}
  = \begin{bmatrix}
  1\\
  1\\
  1 \\
  \end{bmatrix}

  - \begin{bmatrix}
  R \\
  G \\
  B \\
  \end{bmatrix}
$$

式中假设所有的彩色值都归一化到了区间[0,1]内。该式还表名**RGB**可以很容易的使用 1 减去各个 CMY 的值。

由于现实油墨问题，通常需要加入第四种颜色**K**——黑色，于是人们提出了**CMYK**模型。

从 CMY 到 CMYK 模型的转换如下：

$$
  K = min(C,M,Y)
$$

若 K = 1 ，则产生无颜色贡献的颜色，由此得出

$$
  C = 0
$$

$$
M = 0
$$

$$
Y = 0
$$

否则，

$$
  C = (C-K) / (1-K)
$$

$$
  M = (M-K)/(1-K)
$$

$$
  Y = (Y-K)/(1-K)
$$

式中所有的值都在区间[0,1]内，从 CMYK 到 CMY 的转换是：

$$
  C = C(1-K) + K
$$

$$
M = M(1-K) + K
$$

$$
Y = Y(1-K) + K
$$

## HSI 彩色模型

1. 从 RGB 到 HSI 彩色的变换

   $$
   f(x)=\left\{
   \begin{aligned}
    \theta &   & B\le G \\
   360 - \theta &  & B \gt G \\
   \end{aligned}
   \right.
   $$

   式中$\theta$

   $$
    \theta = arccos \left
    \{
      \begin{aligned}
      \frac{\frac{1}{2}[(R-G) + (R-B)]}{[(R-G)^2+(R-B)(G-B)]^{1/2}}
     \end{aligned}
    \right\}
   $$

   饱和度分量为

   $$
    S = 1 - \frac {3} {R+G+B}[min(R,G,B)]
   $$

   亮度分量为

   $$
    I = \frac{1}{3}(R+G+B)
   $$

2. 从 HSI 到 RGB 的彩色变换
   已知 HSI 值在区间[0,1]内，现在我们希望求同一区间的对应 RGB 值。使用公司取决于 H 的值。存在上个我们感兴趣的扇区，他们对于$120 \degree$分割的原色。首先，将 H 乘以$360\degree$使色调值回到[$0\degree$,$360\degree$]内。

   **RG**扇区($0\degree \le H \lt 120 \degree$):当 H 的值在这个扇区时，RGB 分量由下式子给出:

   $$
    B = I(1 - S)
   $$

   $$
    R = I \left [
        \begin{aligned}
         1 + \frac{ScosH}{cos(60\degree - H)}
        \end{aligned}
      \right ]
   $$

   $$
    G = 3I - (R + B)
   $$

   **GB**扇区($120 \degree \le H \lt 240 \degree$):当 H 的值在这个扇区中时，将 H 减去$120 \degree$

   $$
    H = H - 120 \degree
   $$

   $$
    R = I(1 - S)
   $$

   $$
    G = I \left [
        \begin{aligned}
         1 + \frac{ScosH}{cos(60\degree - H)}
        \end{aligned}
      \right ]
   $$

   $$
    B = 3I - (R + G)
   $$

   **BR**扇区($240 \degree \le H \le 360 \degree$):最后，H 的值在这个扇区中时，将 H 减去$240 \degree$

   $$
    H = H - 240 \degree
   $$

   于是 RGB 的分量为

   $$
    G = I(1-S)
   $$

   $$
    B = I \left [
        \begin{aligned}
         1 + \frac{ScosH}{cos(60\degree - H)}
        \end{aligned}
      \right ]
   $$

   $$
   R = 3I - (G + B)
   $$
