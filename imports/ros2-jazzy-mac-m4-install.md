---
title: "Mac M4 虚拟机 Ubuntu 24.04 安装 ROS2 Jazzy 新手教程"
slug: "ros2-jazzy-mac-m4-install-v4"
category: "ROS2"
status: "published"
cover_url: "https://i-blog.csdnimg.cn/direct/5188a3f95f5145c6921ce2dac9fe91e9.png"
tags: "ROS2, Ubuntu, Mac, Jazzy, 虚拟机, 教程"
---


# 1、系统准备
## 1.1、安装 Ubuntu 24.04 LTS
M4虚拟机软件很多，诸如Parallels Desktop（收费）、OrbStack（免费）、UTM（免费）等，桌面仿真推荐PD；终端推荐OrbStack，以下是两款工具的对比，可以按需选择。
| 对比项       | Parallels Desktop（PD）                  | OrbStack                          |
|--------------|------------------------------------------|-----------------------------------|
| 核心定位     | 完整虚拟机，全系统支持                   | 轻量容器 + Linux 终端虚拟机       |
| 桌面 GUI     | ✅ 完整支持（Windows/macOS/Linux 桌面） | ❌ 仅 Linux 终端，无桌面          |
| 启动速度     | 慢（数十秒）                             | 极快（<2 秒）                     |
| 资源占用     | 高（内存 GB 级）                        | 极低（内存 <300MB）               |
| 支持系统     | Windows/macOS/Linux/BSD 全系统          | 仅 Linux（终端）+ Docker 容器     |
| 文件共享     | 无缝融合模式                             | 自动挂载，性能优秀                |
| 网络         | 桥接 / NAT/host 等多种模式              | 自动端口映射、host 网络           |
| 价格         | 商业付费（订阅 / 永久）                  | 个人免费，商业付费                |
| 适用人群     | 全场景、需要桌面、多系统用户             | 开发者、命令行、容器 / DevOps 场景|

（1）PD一键安装 桌面版 Ubuntu 24.04 LTS
![在这里插入图片描述](https://i-blog.csdnimg.cn/direct/f0694f2ac6e34add87925119621c92f5.png)
点击继续、下载即可，
![在这里插入图片描述](https://i-blog.csdnimg.cn/direct/a2da80a96dec440db3f53ab748bd558a.png)
下载完成直接进入系统登录界面
![在这里插入图片描述](https://i-blog.csdnimg.cn/direct/782fdea92f2148928638492e07ec0671.png)
（2）OrbStack一键安装 终端版 Ubuntu 24.04 LTS
![在这里插入图片描述](https://i-blog.csdnimg.cn/direct/efc8dd17f7b14bd398db6e5b3595e6ff.png)
点击 Create，即可开始下载并自动安装
![在这里插入图片描述](https://i-blog.csdnimg.cn/direct/047b31bd876544e7b1abbdb48005f6cc.png)
## 1.2、Ubuntu 系统设置
（1）设置系统编码为 UTF-8
查看系统编码
```bash
locale
echo $LANG		# en_US.UTF-8
```
以上虚拟机默认已经是 UTF-8，
如果其他方式安装的，不存在或系统编码不符合的，可以通过以下命令来安装并进行设置：
```bash
sudo apt update && sudo apt install locales

sudo locale-gen en_US en_US.UTF-8
sudo update-locale LC_ALL=en_US.UTF-8 LANG=en_US.UTF-8
export LANG=en_US.UTF-8
```

（2）设置系统软件源
查看系统架构
```bash
uname -m
aarch64
```
输出结果参考以下表格：
| 架构类型       | 常见输出值                | 适用场景 / 说明                                                         |
|----------------|---------------------------|------------------------------------------------------------------------|
| x86（32 位）   | i386/i486/i586/i686       | 老旧 32 位 x86 处理器（如早期 Intel/AMD 处理器），现代系统几乎不再使用 |
| x86_64（64 位）| x86_64/amd64              | 主流 64 位 x86 架构（Intel Core/AMD Ryzen 处理器），Linux/macOS/Windows 通用 |
| ARM 32 位      | armv6l/armv7l             | 树莓派 1/2/Zero（armv6l）、树莓派 3/4 32 位系统（armv7l），l 表示小端序 |
| ARM 64 位      | aarch64/arm64             | 树莓派 4/5 64 位系统、M1/M2/M3 Mac、ARM 服务器（如 AWS Graviton）、安卓手机 |
| PowerPC        | ppc/ppc64/ppc64le         | IBM Power 服务器，le 表示小端序                                        |
| MIPS           | mips/mips64/mipsel        | 嵌入式设备、路由器（如小米路由器），el 表示小端序                       |
| RISC-V         | riscv64                   | 新兴开源架构（如香橙派 RISC-V 开发板），目前主要是 64 位版本           |
| SPARC          | sparc/sparc64             | Sun/Oracle 服务器，小众架构                                             |
| Alpha          | alpha                     | DEC 老旧服务器架构，几乎淘汰                                           |

因为M4本身是ARM64架构，所以在它上面虚拟的Ubuntu也是ARM64架构的。

设置ARM64位专用的阿里云软件源（对应 ubuntu-ports ）：
```bash
sudo tee /etc/apt/sources.list << 'EOF'
deb http://mirrors.aliyun.com/ubuntu-ports/ noble main restricted universe multiverse
deb http://mirrors.aliyun.com/ubuntu-ports/ noble-updates main restricted universe multiverse
deb http://mirrors.aliyun.com/ubuntu-ports/ noble-security main restricted universe multiverse
deb http://mirrors.aliyun.com/ubuntu-ports/ noble-backports main restricted universe multiverse
EOF
```
附：X86_64位的阿里云源（对应 ubuntu ）
```bash
sudo tee /etc/apt/sources.list << 'EOF'
deb http://mirrors.aliyun.com/ubuntu/ noble main restricted universe multiverse
deb http://mirrors.aliyun.com/ubuntu/ noble-security main restricted universe multiverse
deb http://mirrors.aliyun.com/ubuntu/ noble-updates main restricted universe multiverse
deb http://mirrors.aliyun.com/ubuntu/ noble-proposed main restricted universe multiverse
deb http://mirrors.aliyun.com/ubuntu/ noble-backports main restricted universe multiverse
deb-src http://mirrors.aliyun.com/ubuntu/ noble main restricted universe multiverse
deb-src http://mirrors.aliyun.com/ubuntu/ noble-security main restricted universe multiverse
deb-src http://mirrors.aliyun.com/ubuntu/ noble-updates main restricted universe multiverse
deb-src http://mirrors.aliyun.com/ubuntu/ noble-proposed main restricted universe multiverse
deb-src http://mirrors.aliyun.com/ubuntu/ noble-backports main restricted universe multiverse
EOF
```
注意：以上源一定不能混用，必须要区分核心架构。
更新系统
```bash
sudo apt update
sudo apt upgrade -y
```
# 2、安装 ROS2 jazzy
## 2.1、安装基础工具
```bash
sudo apt install curl gnupg lsb-release
```
## 2.2、添加 ROS 软件源
```bash
sudo curl -sSL https://raw.githubusercontent.com/ros/rosdistro/master/ros.key -o /usr/share/keyrings/ros-archive-keyring.gpg

echo "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/ros-archive-keyring.gpg] http://packages.ros.org/ros2/ubuntu $(source /etc/os-release && echo $UBUNTU_CODENAME) main" | sudo tee /etc/apt/sources.list.d/ros2.list > /dev/null
```
## 2.3、安装 ROS2
```bash
sudo apt update
sudo apt upgrade -y
sudo apt install ros-jazzy-desktop -y			# RViz + rqt + TF2 + 基础库
sudo apt install ros-jazzy-desktop-full -y		# desktop + Gazebo 全套仿真环境
```
做机器人仿真（导航、SLAM、机械臂），必装 full。

## 2.4、设置 ROS2 环境变量
查看当前shell类型
```bash
echo $SHELL
/bin/bash
```
（1）如果你使用 bash
```bash
# 对所有终端有效，写入终端启动前执行的配置文件，每次启动终端自动执行
echo "source /opt/ros/jazzy/setup.bash" >> ~/.bashrc
source ~/.bashrc
```

（2）如果你使用 zsh
```bash
echo "source /opt/ros/jazzy/setup.zsh" >> ~/.zshrc
source ~/.zshrc
```
## 2.5、ROS2 安装验证
```bash
# 查看 ROS2 命令
ros2
```
![在这里插入图片描述](https://i-blog.csdnimg.cn/direct/5188a3f95f5145c6921ce2dac9fe91e9.png)

```bash
# 运行官方示例：消息发布/订阅测试（必须两个终端）
ros2 run demo_nodes_cpp talker      	# 发布消息：Hello World：序号
ros2 run demo_nodes_cpp listener		# 订阅消息：Hello World：序号
```
![在这里插入图片描述](https://i-blog.csdnimg.cn/direct/dcb59e3798d14594bdd774aee848c83b.png)
![在这里插入图片描述](https://i-blog.csdnimg.cn/direct/3fb8b1264b78455784b463f509f9d2fa.png)


```bash
# 小海龟移动测试（必须在桌面端，运行两个终端）
ros2 run turtlesim turtlesim_node		# 显示小海龟
ros2 run turtlesim turtle_teleop_key	# 鼠标控制小海龟
```
![在这里插入图片描述](https://i-blog.csdnimg.cn/direct/4e034c412f764ddcbfbf225fa4c04f3a.png)
至此，ROS2 已经安装成功。
## 2.6、卸载 ROS2
```bash
sudo apt remove ros-jazzy-* -y
sudo apt autoremove -y
sudo rm /etc/apt/sources.list.d/ros2.list
```

# 3、开发环境设置
## 3.1、创建工作空间
工作空间（workspace）是一个存放工程开发相关文件的文件夹。
- src：代码空间（Source Space），功能包里的源码、脚本等。
- build：编译空间（Build Space），编译过程产生的中间文件，不用管。
- install：安装空间（Install Space），编译完成之后产生的可执行文件、脚本。
- log：日志空间（Log Space），编译过程中以及后续运行过程中产生的各种日志信息。
```bash
mkdir -p ~/Workspace/dev_ws/src		# 创建src目录
cd ~/Workspace/dev_ws/src	

# 下载胡春旭老师的教程
git clone https://gitee.com/guyuehome/ros2_21_tutorials.git
```
## 3.2、安装依赖
```bash
sudo apt install -y python3-pip

# 国内工具rosdepc替代rosdep，解决国外网络依赖
sudo pip3 install rosdepc
# 报错则执行以下命令：
sudo pip3 install rosdepc --break-system-packages

# 更新环境
$ sudo rosdepc init 
$ rosdepc update
$ cd ..		# 回退到src上级目录中

# 搜索当前src目录，自动安装所需依赖
$ rosdepc install -i --from-path src --rosdistro jazzy -y
```
## 3.3、编译工作空间
colcon 是一个命令行工具，用于改进编译、测试和使用多个软件包的工作流程。它实现过程自动化，处理需求并设置环境以便于使用软件包。ROS2 中便是使用 colcon 作为包构建工具的，但是 ROS2 中没有默认安装 colcon，需要自行安装，安装命令如下：
```bash
# 首先安装编译工具
sudo apt install python3-colcon-ros

# 在工作空间目录，进行编译
colcon build		# 编译过程生成另外三个目录：build install log
```
## 3.4、设置工作空间环境变量
```bash
echo "source ~/Workspace/dev_ws/install/local_setup.sh" >> ~/.bashrc
```
## 3.5、创建功能包
```bash
cd src
ros2 pkg create <package_name> --build-type <bulid-type> 
```




