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


def fft(x):   
    N = len(x) 
    if N == 1: 
        return x 
    else: 
        X_even = fft(x[::2])
        X_odd = fft(x[1::2])
        factor = np.exp(-2j * np.pi * np.arange(N) / N)
        X = np.concatenate([
            X_even + factor[:int(N / 2)] * X_odd , 
            X_even + factor[int(N / 2) :]  * X_odd
            ])
        return X

def fft2(x):
  h,w = x.shape
  X = np.zeros((h,w))
  for r in h :
    for c in w :
      X[:,c] += fft(x[:,c])
    X[r,:] += fft(x[r,:])
  return X 

  def fft(x):
   
    N = len(x) 
    if N == 1: 
        return x 
    else: 
        X_even = fft(x[::2])
        X_odd = fft(x[1::2])
        factor = np.exp(-2j * np.pi * np.arange(N) / N)
        X = np.concatenate([
            X_even + factor[:int(N / 2)] * X_odd , 
            X_even + factor[int(N / 2) :]  * X_odd
            ])
        return X