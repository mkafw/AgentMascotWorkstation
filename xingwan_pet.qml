import QtQuick
import QtQuick.Window
import QtQuick.Layouts

Window {
    id: root
    visible: true
    title: "xingwan-pet"
    width: 280
    height: 440
    color: "transparent"
    flags: Qt.FramelessWindowHint | Qt.WindowStaysOnTopHint

    // 默认定位在屏幕右下角 (逻辑分辨率 960x540)
    x: Screen.width > 280 ? Screen.width - 290 : 660
    y: Screen.height > 440 ? Screen.height - 450 : 90

    property string currentTask: "TASK-HIST-PRUNE-001: 跨月历史轨迹倒查与脱水瘦身"
    property int taskProgress: 85
    property string bubbleText: "正在执行跨月全量指令逐日倒查与脱水瘦身~ ✨"
    property bool isAlert: false
    property bool isRunning: true

    // 定时读取运行时状态
    Timer {
        interval: 1000
        running: true
        repeat: true
        onTriggered: syncState()
    }

    function syncState() {
        var xhr = new XMLHttpRequest();
        xhr.open("GET", "file:///home/daoxuan/Projects/AgentMascotWorkstation/tasks/runtime_state.json?t=" + Date.now());
        xhr.onreadystatechange = function() {
            if (xhr.readyState === XMLHttpRequest.DONE) {
                if (xhr.status === 200 || xhr.status === 0) {
                    try {
                        var d = JSON.parse(xhr.responseText);
                        if (d.bubble_text) bubbleText = d.bubble_text;
                        if (typeof d.progress === "number") taskProgress = d.progress;
                        var t = d.current_task || d.active_task_id || "无任务";
                        currentTask = t;
                        isAlert = d.needs_human_card || d.alert_human || false;
                        isRunning = (t !== "无任务" && taskProgress < 100);
                    } catch(e) {}
                }
            }
        };
        xhr.send();
    }

    Component.onCompleted: syncState()

    // 允许拖拽移动整个桌宠
    MouseArea {
        id: dragArea
        anchors.fill: parent
        property point clickPos: "0,0"
        onPressed: function(mouse) {
            clickPos = Qt.point(mouse.x, mouse.y)
        }
        onPositionChanged: function(mouse) {
            var delta = Qt.point(mouse.x - clickPos.x, mouse.y - clickPos.y)
            root.x += delta.x
            root.y += delta.y
        }
        onClicked: {
            // 戳一戳互动反馈
            charBounceAnim.restart()
            bubbleText = "叫我把风呀，笨蛋主人~ ✨ (四翅应龙已锁定当前管线)"
        }
    }

    Column {
        anchors.fill: parent
        spacing: 6
        alignment: Qt.AlignHCenter

        // 1. 顶部指示黑板 (任务、红绿灯与实时进度条)
        Rectangle {
            id: blackBoard
            width: parent.width - 20
            height: 110
            anchors.horizontalCenter: parent.horizontalCenter
            color: "#1e1b2e"
            radius: 12
            border.color: isAlert ? "#eab308" : (isRunning ? "#ef4444" : "#22c55e")
            border.width: 1.5
            opacity: 0.95

            Column {
                anchors.fill: parent
                anchors.margins: 8
                spacing: 4

                // 标题行与状态灯
                Row {
                    width: parent.width
                    spacing: 6
                    Rectangle {
                        width: 10
                        height: 10
                        radius: 5
                        color: isAlert ? "#eab308" : (isRunning ? "#ef4444" : "#22c55e")
                        anchors.verticalCenter: parent.verticalCenter
                        // 呼吸发光
                        SequentialAnimation on opacity {
                            loops: Animation.Infinite
                            PropertyAnimation { from: 1.0; to: 0.4; duration: 800 }
                            PropertyAnimation { from: 0.4; to: 1.0; duration: 800 }
                        }
                    }
                    Text {
                        text: isAlert ? "● 卡点求助 (需拍板)" : (isRunning ? "● 正在执行 (RUNNING)" : "● 空闲就绪 (IDLE)")
                        color: isAlert ? "#fde047" : (isRunning ? "#fca5a5" : "#86efac")
                        font.pixelSize: 11
                        font.bold: true
                    }
                    Item { width: 1; height: 1; Layout.fillWidth: true }
                    Text {
                        text: taskProgress + "%"
                        color: "#38bdf8"
                        font.pixelSize: 11
                        font.bold: true
                    }
                }

                // 当前任务名称
                Text {
                    width: parent.width
                    text: currentTask
                    color: "#f8fafc"
                    font.pixelSize: 11
                    font.bold: true
                    elide: Text.ElideRight
                }

                // 进度条背景与填充
                Rectangle {
                    width: parent.width
                    height: 6
                    radius: 3
                    color: "#0f172a"
                    clip: true

                    Rectangle {
                        width: parent.width * (Math.min(100, Math.max(0, taskProgress)) / 100.0)
                        height: parent.height
                        radius: 3
                        color: isAlert ? "#eab308" : (isRunning ? "#ef4444" : "#38bdf8")
                    }
                }

                // 气泡台词摘要
                Text {
                    width: parent.width
                    text: "「" + bubbleText + "」"
                    color: "#94a3b8"
                    font.pixelSize: 10
                    elide: Text.ElideRight
                }
            }
        }

        // 2. 正面照角色主体 (100% 纯透明无底色)
        Item {
            width: parent.width
            height: 310
            anchors.horizontalCenter: parent.horizontalCenter

            Image {
                id: petSprite
                anchors.fill: parent
                fillMode: Image.PreserveAspectFit
                // 强制锁定纯正面立绘
                source: "file:///home/daoxuan/Projects/AgentMascotWorkstation/assets/xingwan_transparent/chibi_front.png"
                smooth: true

                // 呼吸轻微浮动
                SequentialAnimation on y {
                    loops: Animation.Infinite
                    PropertyAnimation { from: 0; to: -5; duration: 1800; easing.type: Easing.InOutQuad }
                    PropertyAnimation { from: -5; to: 0; duration: 1800; easing.type: Easing.InOutQuad }
                }
            }

            // 点击 Q 弹动画
            SequentialAnimation {
                id: charBounceAnim
                PropertyAnimation { target: petSprite; property: "scale"; from: 1.0; to: 1.15; duration: 100; easing.type: Easing.OutQuad }
                PropertyAnimation { target: petSprite; property: "scale"; from: 1.15; to: 0.95; duration: 120; easing.type: Easing.InOutQuad }
                PropertyAnimation { target: petSprite; property: "scale"; from: 0.95; to: 1.0; duration: 100; easing.type: Easing.InQuad }
            }
        }
    }
}
