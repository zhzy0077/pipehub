import React, { CSSProperties, useContext, useState } from 'react';
import { Stack, FontSizes, Icon, Text, Label, TextField, PrimaryButton } from '@fluentui/react';
import User from './User';

interface SendProps {
    user: User,
}

function Send(props: SendProps) {
    const [message, setMessage] = useState("");
    const send = () => {
        fetch(`${props.user.callback_url}`, {
            method: "POST",
            body: message,
        })
            .then(res => {
                return res.json();
            }).then(entity => {
                const str = JSON.stringify(entity, null, 2);
                alert(str);
            });
    };

    const onMessageUpdate = (event: React.FormEvent<HTMLInputElement | HTMLTextAreaElement>, newVal?: string) => {
        setMessage(newVal || "");
    }

    return (
        <Stack gap={15}>
            <Stack.Item>
                <Label>Payload</Label>
            </Stack.Item>
            <Stack.Item>
                <TextField multiline rows={3} onChange={onMessageUpdate} value={message} />
            </Stack.Item>
            <Stack.Item>
                <PrimaryButton onClick={send}>测试发送</PrimaryButton>
            </Stack.Item>
        </Stack>
    );
}

export default Send;
