import React from 'react';
import { Depths } from '@uifabric/fluent-theme/lib/fluent/FluentDepths';
import { Stack, Label, FontSizes, Text, Link } from '@fluentui/react';

function Faq() {
  return (
    <Stack tokens={{ padding: '20px' }} style={{ boxShadow: Depths.depth4 }}>
      <Stack.Item>
        <div>
          <Label style={{ fontSize: FontSizes.xLarge }} >FAQ</Label>
          <Label>PipeHub 有什么用?</Label>
          <Text block>
            简单来说就是一个方便你实现推送通知的玩意, 你可以通过简单的 HTTP 调用来将各种消息发送到手机上. 目前我用来做:
          </Text>
          <Text block style={{ marginTop: '5px' }}>
            1. 对于 Android 用户来说, 原生, 国产 Rom 和不折腾几乎是个不可能三角, 我通过 PipeHub 来推送 Google FCM 消息, 以避免在国产 Rom 上折腾FCM.
          </Text>
          <Text block style={{ marginTop: '5px' }}>
            2. 将一张专门用来收验证码的手机号放在旧手机中, 通过 PipeHub 推送收到的短信, 避免占用一个卡槽并且把我的短信箱搞的一团糟.
          </Text>
          <Text block style={{ marginTop: '5px' }}>
            3. 通过一些 IFTTT 的规则, 推送一些消息, 如 Breaking News, 天气预报等.
          </Text>
          <Label style={{ marginTop: '10px' }}>为啥要做 PipeHub ?</Label>
          <Text block>
            我个人强依赖上述的几个用法, 就做了这个玩意, 想到可能也有人有和我类似的需求, 就买了个域名搭了这个网站, 希望能帮到谁吧.
          </Text>
          <Label style={{ marginTop: '10px' }}>PipeHub 和 Server 酱有啥区别?</Label>
          <Text block>
            我以前也是 Server 酱的用户, 后来 Server 酱改成模板消息后就收不到推送很苦恼, 就做了PipeHub. PipeHub 使企业微信进行推送, 可靠性更好的也更安全.
          </Text>
          <Label style={{ marginTop: '10px' }}>PipeHub 收费吗?</Label>
          <Text block>
            免费, 不出意外的话 PipeHub 应该会一直免费下去.
          </Text>
          <Label style={{ marginTop: '10px' }}>PipeHub 开源吗?</Label>
          <Text block>
            目前开源在: <Link href="https://github.com/zhzy0077/PipeHub">https://github.com/zhzy0077/PipeHub</Link>, 欢迎 Star.
          </Text>
          <Label style={{ marginTop: '10px' }}>我的数据在哪里?</Label>
          <Text block>
            隐私是每个人最重要的权利之一, PipeHub 通过以下方式保障您的隐私:
          </Text>
          <Text block style={{ marginTop: '5px' }}>
            1. PipeHub 仅依赖企业微信, 您可以(这也是推荐的方式)注册一个单独的企业用于推送, PipeHub 除了可以向您推送消息以外对您一无所知.
          </Text>
          <Text block style={{ marginTop: '5px' }}>
            2. PipeHub 在 GitHub 登录过程中不要求任何权限, 也就是说 PipeHub 并不比一个不小心点开您 GitHub 主页的人知道更多.
          </Text>
          <Text block style={{ marginTop: '5px' }}>
            3. PipeHub 不会存储 / 记录您发送的任何一条消息, 仅会对您是否发送成功做日志性的记录.
          </Text>
        </div>
      </Stack.Item>
    </Stack >
  );
}

export default Faq;
