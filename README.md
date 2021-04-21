# silicon-api
a simple api transform your code to beautiful picture.

## 使用方法

```http request
POST https://api.wdvxdr.com/silicon
Content-Type: application/json

{
    "code": "fn main() {\n\tprintln!(\"Hello world!\");\n}",
    "format": {
        "language":"txt",
        "theme":"Dracula",
        "line_pad": 2,
        "line_offset":1,
        "tab_width":4
    }
}
```

## 部署

从 actions 下载， linux下需要依赖

```bash
sudo apt install expat
sudo apt install libxml2-dev
sudo apt install pkg-config libasound2-dev libssl-dev cmake libfreetype6-dev libexpat1-dev libxcb-composite0-dev
sudo apt install fontconfig
```