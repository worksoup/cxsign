## 签到时的滑块验证

（应该仅限部分二维码签到？这点并不能确定，因为这部分逻辑是服务器返回的。）

1. 签到后，除返回 success 和错误信息外，现在还会返回 `validate_XXX` 类的消息。
   其中 XXX 被称为 `enc2`, 格式也与 `enc` 类似。
2. 如果 `enc2` 存在，则 XXT 会调用一个方法，初始化 CXCaptcha.
3. 滑块验证成功后会请求 `url + "&enc2" + enc2 + "&validate=validate_" + captcha_id +"_" + token`.
   其中 `url` 是 `pptSign` 的带参数的 url, `enc2` 见上。`captcha_id` 似乎是个固定值，而 `token` 是
   滑块验证之前就可以获取的一个参数，下文详述。

目前不知道：

- 滑块阶段算不算在计算二维码过期的时间之内。
- 不进行滑块验证是否可以签到（获取 `token` 后直接请求而不进行滑块验证，不过大概率不行）。
- `token` 是否可以多次使用，即能否获取一次 `token` 后重复使用。
- `token` 是否与签到对应，即能否在网页上操作一次获取 `token` 后签到（这个有可能）。
- 滑块验证是会收集浏览器操作数据以判断是否为及其操作还是单纯滑对位置（以及判断请求头）即可。

## 滑块的大致生成逻辑

1. 本地缓存的服务器时间如果和当前时间差值超过 4 分钟则刷新一次本地的服务器时间。
2. 通过服务器时间生成 `key` 和 `tmp_token`. 其中 `key` 是 `hash(server_time as str + rand_uuid)`,
   `tmp_token` 是
   `hash(server_time as str + captcha_id + "slide" + key) + ":" + (server_time + 300000) as str`
   这个生成`rand_uuid`的函数和哈希函数能看到源码。生成一个十六进制字符串，类似于 MD5 但是似乎不是。
3. ```
   GET/ url=GET_CAPTCHA,
   dataType="jsonp" //所以请求的 url 里会出现一个 `callback={callback-name}`. // ?
   data: {
     captchaId: captcha_id,
     type: "slide",
     version: "1.1.16",
     captchaKey: key,
     token: tmp_token,
     referer: location.href,
   }.
   响应格式为：
   callback_name(
     {
       "token":token,
       "imageVerificationVo":
       {
         "type":"slide",
         "shadeImage": shade_image_url,
         "cutoutImage":cutout_image_url
       }
     }
   )
   ```
   这里获取到了 `token`, 此时还未进行滑块。
4. ```
   textClickArr = Json.stringify([{"x":number}])
   coordinate = Json.stringify([])
   enum runEnv:
      WEB=> 10,
      ANDROID=> 20,
      IOS=> 30,
      MINIPROGRAM=> 40
   ```

   ```
   GET/ url=CHECK_CAPTCHA, dataType="jsonp"
   data:{captchaID,type:"slide",textClickArr,coordinate,runEnv:10,version:"1.1.16"}
   ```

   响应格式为：

   ```
   callback_name(
   {
   "error":0,
   "msg":"ok",
   "result":true,
   "extraData":validate_str
   }
   )
   ```

   若 `error` 不为 0 则 "msg" 为错误信息，其余两项不存在。
   `extraData` 是一个 JSON 格式的字符串。
   见 XXT 的注释：

   ```
   /**
     * ...
     * 第二个参数是data对象，验证成功后的相关信息，data数据结构为key-value，如下：
     * {
     * validate: 'xxx' // 二次验证信息
     * }
     **/
   ```

   其中 `validate` 与 `"validate_" + captcha_id +"_" + token` 一致。
   目前不知到滑块验证成功的判定条件是什么。

## 注

```
static CAPTCHA_ID: &str = "Qt9FIw9o4pwRjOyqM6yizZBh682qN2TU";
// 获取滑块。
static GET_CAPTCHA: &str = "https://captcha.chaoxing.com/captcha/get/verification/image";
// 滑块验证。
static CHECK_CAPTCHA: &str = "https://captcha.chaoxing.com/captcha/check/verification/result";
// 获取服务器时间。
static GET_SERVER_TIME: &str = "https://captcha.chaoxing.com/captcha/get/conf";
```
