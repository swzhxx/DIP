import numpy as np

def color_normalize(img):
    _img = np.abs(img)
    mmax = np.max(_img)
    mmin = np.min(_img)
    mrange = mmax - mmin
    h,w = img.shape 
    normialized_img = np.zeros((h,w))
    for i in range(h):
        for j in range(w):
            normialized_img[i][j] = 255 * ((_img[i][j] - mmin) / mrange)
    return normialized_img


def sp_noise(image,prob):
    '''
    添加椒盐噪声
    prob:噪声比例 
    '''
    output = np.zeros(image.shape,np.uint8)
    thres = 1 - prob 
    for i in range(image.shape[0]):
        for j in range(image.shape[1]):
            rdn = random.random()
            if rdn < prob:
                output[i][j] = 0
            elif rdn > thres:
                output[i][j] = 255
            else:
                output[i][j] = image[i][j]
    return output


def gauss_noise(img,sigma):
	temp_img = np.float64(np.copy(img))
	h = temp_img.shape[0]
	w = temp_img.shape[1]
	noise = np.random.randn(h,w) * sigma
	noisy_img = np.zeros(temp_img.shape, np.float64)
	if len(temp_img.shape) == 2:
		noisy_img = temp_img + noise
	else:
		noisy_img[:,:,0] = temp_img[:,:,0] + noise
		noisy_img[:,:,1] = temp_img[:,:,1] + noise
		noisy_img[:,:,2] = temp_img[:,:,2] + noise
	# noisy_img = noisy_img.astype(np.uint8)
	return noisy_img


def img_cretralization(img):
    h,w = img.shape
    for y in range(h):
        for x in range(w):
            img[x][y] = img[x][y] * (-1) ** (x+y)


def motion_blur(img, a,b,T):
  h,w = img.shape 
  H = np.zeros((h,w) ,dtype="complex")
  p = w / 2 + 1 
  q = h / 2 + 1 
  _img = img.copy()
  img_cretralization(_img)
  for v in range(h):
      for u in range(w):
          temp = np.pi * ((u - p) * a + (v - q) * b)
          if temp == 0 :
              H[u,v] = T
          else :
              H[u,v] = T * np.sin(temp) * np.exp(-1j *(temp)) /  (temp)
  F = np.fft.fft2(_img)
  G = H * F 
  g = np.fft.ifft2(G)
  g = np.real(g)
  img_cretralization(g)
  return g  , H

