# 超星签到 api 整理
## 登陆页
- 链接：`http://passport2.chaoxing.com/mlogin?fid=&newversion=true&refer=http%3A%2F%2Fi.chaoxing.com`
- 方法：`GET`
- 字段：无
## 明文密码登陆
- 链接： `https://passport2-api.chaoxing.com/v11/loginregister`
- 方法：`GET`
- 字段：
  - `code=${pwd}`
  - `cx_xxt_passport=json`
  - `uname=${uname}`
  - `loginType=1`
  - `roleSelect=true`
- 返回：`cookie`, `uid`, `{"mes":"验证通过","type":1,"url":"https://sso.chaoxing.com/apis/login/userLogin4Uname.do","status":true}`
## 非明文密码登陆
- 链接：`http://passport2.chaoxing.com/fanyalogin`
- 方法：`POST`
- 字段：
  - `fid=-1`
  - `uname=${uname}`
  - `password=${encryptPwd}`
  - `refer=https%253A%252F%252Fwww.baidu.com%252Flink%253Furl%253D7F6K1ISfp_Qh_YMOftV_1CdfwkA8zQnhOR6jlqtCVZxdMssUZVIX2uVSC1NXiSebyQ8Ur8YILmFm0Vo7naeSl_%2526wd%253D%2526eqid%253Df3f8f74c0023361200000003634e4a8b`
  - `t=true`
  - `forbidotherlogin=0`
  - `validate=`
  - `doubleFactorLogin=0`
  - `independentId=0`
- 说明：`encryptPwd` 是 `DES` 加密后的密码。加密时：
  - `mode: ECB`
  - `padding: Pkcs7`
  - `iv: u2oh6Vu^HWe40fj`
- 返回：`cookie`, `uid`
## 预签到
- 链接：`https://mobilelearn.chaoxing.com/newsign/preSign`
- 方法：`GET`
- 字段：
  - `courseId=${courseId}`
  - `classId=${classId}`
  - `activePrimaryId=${activeId}`
  - `general=1`
  - `sys=1`
  - `ls=1`
  - `appType=15`
  - `tid=`
  - `uid=${uid}`
  - `ut=s`
- 说明：所有签到接口访问前都必须进行一次预签到。
## 签到
- 链接：`https://mobilelearn.chaoxing.com/pptSign/stuSignajax`
- 方法：`GET`
- 字段：见后签到方式详述部分。
- 说明：适用于普通签到，手势签到，签到码签到，拍照签到，位置签到。
## 签到信息获取
- 链接：`https://mobilelearn.chaoxing.com/v2/apis/active/getPPTActiveInfo`
- 方法：`GET`
- 字段：
  - `activeId=${activeId}`
- 说明：获取签到手势或签到码等。
## 获取课程
- 链接：`http://mooc1-1.chaoxing.com/visit/courselistdata`
- 方法：`POST`
- 字段：
```
  const formdata = 'courseType=1&courseFolderId=0&courseFolderSize=0';
  const result = await request(
    COURSELIST.URL,
    {
      gzip: true,
      method: COURSELIST.METHOD,
      headers: {
        Accept: 'text/html, */*; q=0.01',
        'Accept-Encoding': 'gzip, deflate',
        'Accept-Language': 'zh-CN,zh;q=0.9,en;q=0.8,en-GB;q=0.7,en-US;q=0.6',
        'Content-Type': 'application/x-www-form-urlencoded; charset=UTF-8;',
        Cookie: `_uid=${_uid}; _d=${_d}; vc3=${vc3}`,
      },
    },
    formdata
  );
```
## 获取课程（`chaoxing-sign-cli` 并未使用）
- 链接：`http://mooc1-api.chaoxing.com/mycourse/backclazzdata`
- 方法：`GET`
- 字段：
  - `view=json`
  - `rss=1`
- 返回：返回一个 json 对象。班级对象的 `key` 字段或 `course` 字段的 `id` 字段是所谓 classId, 班级对象的 `course` 字段的 `data` 数组中则是课程对象，课程对象的 `id` 字段即为所谓 courseId.
## 查询活动 1
- 链接：`https://mobilelearn.chaoxing.com/v2/apis/active/student/activelist`
- 方法：`GET`
- 字段：
  - `fid=0`
  - `courseId=${courseId}`
  - `classId=${classId}`
  - `showNotStartedActive=0`
  - `_=1663752482576`
- 说明：最后一个字段是当前时间，但是似乎没有作用。需要进一步研究。
## 查询活动 2
- 链接：`https://mobilelearn.chaoxing.com/ppt/activeAPI/taskactivelist`
- 方法：`GET`
- 字段：
  - `courseId=${courseId}`
  - `classId=${classId}`
- 说明 `User-Agent: Mozilla/5.0 (iPhone; CPU iPhone OS 16_0_3 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Mobile/15E148 (schild:eaf4fb193ec970c0a9775e2a27b0232b) (device:iPhone11,2) Language/zh-Hans com.ssreader.ChaoXingStudy/ChaoXingStudy_3_6.0.2_ios_phone_202209281930_99 (@Kalimdor)_1665876591620212942`
## 账号设置页
- 链接：`http://passport2.chaoxing.com/mooc/accountManage`
- 方法：`GET`
- 字段：无
## 超星网盘页
- 链接：`https://pan-yz.chaoxing.com`
- 方法：`GET`
- 字段：无
- 返回：网盘 `enc`, `parentId`.
## 网盘列表
- 链接：`https://pan-yz.chaoxing.com/opt/listres`
- 方法：`POST`
- 字段：
  - `puid=0`
  - `shareid=0`
  - `parentId=${parentId}`
  - `page=1`
  - `size=50`
  - `enc=${enc}`
- 返回：`objectId`
## 获取超星云盘的 token
- 链接：`https://pan-yz.chaoxing.com/api/token/uservalid`
- 方法：`GET`
- 字段：无
## 网盘上传接口
- 链接：`https://pan-yz.chaoxing.com/upload`
- 方法：`POST`
- 字段：
  - `_from=mobilelearn`
  - `_token=${token}`
- 返回：`objectId`
- 说明： `'Content-Type': 'multipart/form-data;boundary=${form.getBoundary()}'`
## web 聊天页
- 链接：`https://im.chaoxing.com/webim/me`
- 方法：`GET`
- 字段：无
## 无课程群聊的预签到
- 链接：`https://mobilelearn.chaoxing.com/sign/preStuSign`
- 方法：`GET`
- 字段：
## 无课程群聊的签到
- 链接：`https://mobilelearn.chaoxing.com/sign/stuSignajax`
- 方法：`GET`
- 字段：
  - `activeId=${activeId}`
  - `code=`
  - `uid=${uid}`
  - `courseId=null`
  - `classId=0`
  - `general=0`
  - `chatId=${chatId}`
  - `appType=0`
  - `tid=${tuid}`
  - `atype=null`
  - `sys=0`
# 签到细节说明
## 关于签到类型
```
    case 0:
      if (iptPPTActiveInfo.ifphoto === 1) {
        return '拍照签到'; 
      } else {
        return '普通签到'; 
      }
    case 2: return '二维码签到';
    case 3: return '手势签到';
    case 4: return '位置签到';
    case 5: return '签到码签到';
    default: return '未知';
```
## 通用签到
- 课程签到字段：
  - `activeId=${activeId}`
  - `uid=${uid}`
  - `clientip=`
  - `latitude=-1`
  - `longitude=-1`
  - `appType=15`
  - `fid=${fid}`
  - `name=${name}`
- 群聊签到字段：
  - `activeId=${activeId}`
  - `uid=${uid}`
  - `clientip=`
## 位置签到
- 课程签到字段：
  - `name=${name}`
  - `address=${address}`
  - `activeId=${activeId}`
  - `uid=${uid}`
  - `clientip=`
  - `latitude=${lat}`
  - `longitude=${lon}`
  - `fid=${fid}`
  - `appType=15`
  - `ifTiJiao=1`
- 群聊签到（此处需要 `POST` 方法）字段：
  `'Content-Type': 'application/x-www-form-urlencoded; charset=UTF-8'`
  - `address=${encodeURIComponent(address)}`
  - `activeId=${activeId}`
  - `uid=${uid}`
  - `clientip=`
  - `useragent=`
  - `latitude=${lat}`
  - `longitude=${lon}`
  - `fid=`
  - `ifTiJiao=1`
## 拍照签到
- 课程签到字段：
  - `activeId=${activeId}`
  - `uid=${uid}`
  - `clientip=`
  - `useragent=`
  - `latitude=-1`
  - `longitude=-1`
  - `appType=15`
  - `fid=${fid}`
  - `objectId=${objectId}`
  - `name=${encodeURIComponent(name)}`
- 群聊签到字段：
  - `activeId=${activeId}`
  - `uid=${uid}`
  - `clientip=`
  - `useragent=`
  - `latitude=-1`
  - `longitude=-1`
  - `fid=0`
  - `objectId=${objectId}`
## 二维码签到
- 课程签到字段：
  - `enc=${enc}`
  - `name=${name}`
  - `activeId=${activeId}`
  - `uid=${uid}`
  - `clientip=`
  - `location={"result":"1","address":"${address}","latitude":${lat},"longitude":${lon},"altitude":${altitude}}`
  - `latitude=-1`
  - `longitude=-1`
  - `fid=${fid}`
  - `appType=15`
## 手势或签到码签到
同通用签到，但是需要 `signCode` 字段。