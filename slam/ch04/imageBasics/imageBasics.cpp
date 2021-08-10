#include <iostream>
#include <chrono>
using namespace std ; 

#include <opencv2/core/core.hpp>
#include <opencv2/highgui/highgui.hpp>

int main(int argc , char ** argv){
  cv::Mat image; 
  image = cv::imread(argv[1]);
  if(image.data ==nullptr){
    cerr << "文件" <<argv[1] <<"不存在"<<endl;
    return 0;
  }
  cout <<"图像宽" <<image.cols <<"高"<<image.rows<<"通道数"<<image.channels()<<endl;
  cv::imshow("image" , image);
  cv::waitKey(0);
   // 判断image的类型
  if (image.type() != CV_8UC1 && image.type() != CV_8UC3) {
    // 图像类型不符合要求
    cout << "请输入一张彩色图或灰度图." << endl;
    return 0;
  }

  // 图像遍历
  for (size_t y = 0; y<image.rows;y++){
    unsigned char *row_ptr = image.ptr<unsigned char>(y);
    for(size_t x = 0 ; x < image.cols ;x++){
      unsigned char *data_ptr = &row_ptr[x * image.channels()];
      for(int c = 0 ; c !=image.channels();c++){
        unsigned char data = data_ptr[c];
      }
    }
  }

  cv::Mat image_another = image ; 
  image_another(cv::Rect(0,0,100,100)).setTo(0);
  cv::imshow("image" , image);
  cv::waitKey(0);
  cv::destroyAllWindows();
  return 0;
}