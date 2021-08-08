#include <iostream>
using namespace std;

#include <ctime> 
#include <Eigen/Core>
#include <Eigen/Dense>

using namespace Eigen;

#define MATRIX_SIZE 50 

int main(int argc ,char **argv){
  cout<<"USE EIGEN!"<<endl;
  Matrix<float ,2,3> matrix_23;
  Vector3d v_3d; 
  Matrix<float , 3,1>  vd_3d;

  Matrix3d matrix_33 = Matrix3d::Zero();

  Matrix<double, Dynamic , Dynamic>  matrix_dynamic; 
  MatrixXd matrix_x ; 

  //输入数据初始化
  matrix_23 << 1,2,3,4,5,6;
  cout << "matrix 2*3 from 1 to 6 :\n" << matrix_23 << endl;


  //访问元素
  cout<< "print matrix 2*3:"<<endl; 
  for (int i = 0 ; i<2;i++){
    for (int j = 0 ; j<3;j++){
      cout << matrix_23(i , j) << "\t";
    }
    cout<<endl;
  }

  v_3d << 3,2,1;
  vd_3d << 4,5,6;


  matrix_33 = Matrix3d::Random();
  cout <<"random matrix :\n" << matrix_33 << endl ; 
  cout << "transpose :\n" << matrix_33.transpose() << endl ; 
  cout << "sum: "<< matrix_33.sum()<<endl;
  cout << "trace : " << matrix_33.trace() << endl ; 
  cout << "times 10 : \n" << 10 * matrix_33 << endl;
  cout << "inverse : \n" << matrix_33.inverse() << endl;
  cout << "det:" << matrix_33.determinant()<<endl;


  //特征值
  SelfAdjointEigenSolver<Matrix3d> eigen_solver(matrix_33.transpose() * matrix_33);
  cout << "Eigen values = \n" << eigen_solver.eigenvalues() << endl;
  cout << "Eigen vectors = \n" << eigen_solver.eigenvectors() << endl;
 

  return 0;
}