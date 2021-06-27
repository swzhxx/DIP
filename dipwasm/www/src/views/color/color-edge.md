# 彩色边缘检测

边缘检测对图像分割来说是一个重要的工具。

令 r,g,b 是沿 RGB 彩色空间的 R，G，B 轴的单位向量，并定义向量：

$$
  u = \frac {\partial R}{\partial x} r + \frac{\partial G}{\partial x} g + \frac{\partial B} {\partial x} b
$$

和

$$
  v = \frac {\partial R}{\partial y}r + \frac{\partial G}{\partial y}g + \frac{\partial B}{\partial y} b
$$

根据这些向量的点积，令$g_{xx}$,$g_{yy}$,$g_{xy}$为：

$$
  g_{xx} = u \cdot u = u^Tu = |\frac{\partial R}{\partial x}|^2 + |\frac{\partial G}{\partial x}|^2 + |\frac{\partial B}{\partial x}|^2
$$

$$
  g_{yy} = v \cdot v = v^Tv = |\frac{\partial R}{\partial y}|^2 + |\frac{\partial G}{\partial y}|^2 + |\frac{\partial B}{\partial y}|^2
$$

$$
  g_{xy} = u \cdot v = u^Tv = \frac{\partial R}{\partial x}\frac{\partial R}{\partial y} + \frac{\partial G}{\partial x}\frac{\partial G}{\partial y} + \frac{\partial B}{\partial x}\frac {\partial B}{\partial y}
$$

记住，R，G，B 及由此得到的 g 项是 x 和 y 的函数，使用这种表示法可以证明最大变化率的方向可由如下角度给出:

$$
  \theta ({x , y}) = \frac{1}{2}arctan[\frac{2g_{xy}}{g_{xx} - g_{yy}}]
$$

坐标(x,y)出$\theta ({x,y})$方向的变化率的值为：

$$
  F_{\theta(x,y)} = [\frac{1}{2} [g_{xx} + g_{yy}] + (g_{xx} - g_{yy})cos2\theta(x,y) + 2g_{xy}sin2\theta(x,y)]^{1/2}
$$
