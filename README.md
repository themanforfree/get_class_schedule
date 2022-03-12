# Get_Class_Schedule 
将河南工业大学的课表导出至 CSV 文件，可以将其导入至日历，便于日程安排。  

# 特性
- 从环境变量中获取学号和密码
- 从命令行参数中获取学号和密码
- 将教务系统中时间范围记录转换为每节课单独记录

# 使用
通过命令行参数  
``` bash
get_class_schedule -u <USERNAME> -P <PASSWORD>
```
  
通过环境变量
``` bash
export JWGLXT_USERNAME=<USERNAME>
export JWGLXT_PASSWORD=<PASSWORD>
get_class_schedule
```