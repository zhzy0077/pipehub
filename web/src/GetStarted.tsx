import React from 'react';
import { Depths } from '@uifabric/fluent-theme/lib/fluent/FluentDepths';
import { Stack, Label, FontSizes, Text, Link } from '@fluentui/react';

function GetStarted() {
  return (
    <Stack tokens={{ padding: '20px' }} style={{ boxShadow: Depths.depth4 }}>
      <Stack.Item>
        <div>
          <Label style={{ fontSize: FontSizes.xLarge }} >开始</Label>
          <Label>1. 注册一个企业微信号.</Label>
          <Text block>
            在<Link href='https://work.weixin.qq.com/wework_admin/register_wx'>这里</Link>注册一个企业微信. 可以随意填写, 不需要验证账号.
          </Text>
          <Text block>
            记录下在'我的企业' Tab 里看到的企业 ID.
          </Text>
          <Label style={{ marginTop: '10px' }}>2. 关注你的企业微信服务号.</Label>
          <Text block>
            用你的个人微信账号, 在<Link href='https://work.weixin.qq.com/wework_admin/frame#profile/wxPlugin'>微信插件</Link>中关注你的企业微信服务号.
          </Text>
          <Label style={{ marginTop: '10px' }}>3. 创建一个企业微信应用.</Label>
          <Text block>
            在<Link href='https://work.weixin.qq.com/wework_admin/frame#apps'>应用管理</Link>中创建一个应用, 并记录下 Agent ID 和 Secret.
          </Text>
          <Label style={{ marginTop: '10px' }}>4. 大功告成.</Label>
          <Text block>
            在右上角通过 GitHub 登录后, 填上你之前得到的企业 ID, Agent ID 和 Secret 并更新后, 就可以通过 User 页面中的 Callback URL 发送消息了. 请求示例:
          </Text>
          <Text block style={{ marginTop: '5px' }}>
            1. GET https://pipehub.net/send/abcde?text=helloworld.
          </Text>
          <Text block style={{ marginTop: '5px' }}>
            1. POST https://pipehub.net/send/abcde. 在 Payload 中的所有内容都会被推送.
          </Text>
        </div>
      </Stack.Item>
    </Stack >
  );
}

export default GetStarted;
