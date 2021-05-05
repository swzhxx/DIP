import numpy as  np
def dft(x):
  N = len(x)
  n = np.arange(N)
  k = n.reshape((N,1))
  e = np.exp(-2j*np.pi*k*n/N)
  X = np.dot(e , x)
  return X

def idft(x):
  N = len(x)
  n = np.arange(N)
  k = n.reshape((N,1))
  e = np.exp(2j*np.pi*k*n/N)
  X = 1/N * np.dot(e , x)
  return X

def dft_matrix(N):
	i,j = np.meshgrid(np.arange(N),np.arange(N))
	omega = np.exp(-2j*np.pi/N)
	w = np.power(omega,i*j)
	return w

def dft2(image):
	h,w = image.shape[:2]
	output = np.zeros((h,w),dtype=complex)
	output = dft_matrix(h).dot(image).dot(dft_matrix(w))
	return output



  
