import 'package:auto_size_text/auto_size_text.dart';
import 'package:debounce_throttle/debounce_throttle.dart';
import 'package:flutter/material.dart';
import 'package:flutter_hbb/common.dart';
import 'package:flutter_hbb/models/platform_model.dart';
import 'package:get/get.dart';
import 'package:provider/provider.dart';

import '../../consts.dart';
import '../../desktop/widgets/tabbar_widget.dart';
import '../../models/chat_model.dart';
import '../../models/model.dart';
import 'chat_page.dart';

class DraggableChatWindow extends StatelessWidget {
  const DraggableChatWindow(
      {Key? key,
      this.position = Offset.zero,
      required this.width,
      required this.height,
      required this.chatModel})
      : super(key: key);

  final Offset position;
  final double width;
  final double height;
  final ChatModel chatModel;

  @override
  Widget build(BuildContext context) {
    if (draggablePositions.chatWindow.isInvalid()) {
      draggablePositions.chatWindow.update(position);
    }
    return isIOS
        ? IOSDraggable(
            position: draggablePositions.chatWindow,
            chatModel: chatModel,
            width: width,
            height: height,
            builder: (context) {
              return Column(
                children: [
                  _buildMobileAppBar(context),
                  Expanded(
                    child: ChatPage(chatModel: chatModel),
                  ),
                ],
              );
            },
          )
        : Draggable(
            checkKeyboard: true,
            checkScreenSize: true,
            position: draggablePositions.chatWindow,
            width: width,
            height: height,
            chatModel: chatModel,
            builder: (context, onPanUpdate) {
              final child = Scaffold(
                resizeToAvoidBottomInset: false,
                appBar: CustomAppBar(
                  onPanUpdate: onPanUpdate,
                  appBar: (isDesktop || isWebDesktop)
                      ? _buildDesktopAppBar(context)
                      : _buildMobileAppBar(context),
                ),
                body: ChatPage(chatModel: chatModel),
              );
              return Container(
                  decoration:
                      BoxDecoration(border: Border.all(color: MyTheme.border)),
                  child: child);
            });
  }

  Widget _buildMobileAppBar(BuildContext context) {
    return Container(
      color: Theme.of(context).colorScheme.primary,
      height: 50,
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Padding(
              padding: const EdgeInsets.symmetric(horizontal: 15),
              child: Text(
                translate("Chat"),
                style: const TextStyle(
                    color: Colors.white,
                    fontFamily: 'WorkSans',
                    fontWeight: FontWeight.bold,
                    fontSize: 20),
              )),
          Row(
            crossAxisAlignment: CrossAxisAlignment.center,
            children: [
              IconButton(
                  onPressed: () {
                    chatModel.hideChatWindowOverlay();
                  },
                  icon: const Icon(
                    Icons.keyboard_arrow_down,
                    color: Colors.white,
                  )),
              IconButton(
                  onPressed: () {
                    chatModel.hideChatWindowOverlay();
                    chatModel.hideChatIconOverlay();
                  },
                  icon: const Icon(
                    Icons.close,
                    color: Colors.white,
                  ))
            ],
          )
        ],
      ),
    );
  }

  Widget _buildDesktopAppBar(BuildContext context) {
    return Container(
      decoration: BoxDecoration(
          border: Border(
              bottom: BorderSide(
                  color: Theme.of(context).hintColor.withOpacity(0.4)))),
      height: 38,
      child: Row(
        mainAxisAlignment: MainAxisAlignment.spaceBetween,
        children: [
          Padding(
              padding: const EdgeInsets.symmetric(horizontal: 15, vertical: 8),
              child: Obx(() => Opacity(
                  opacity: chatModel.isWindowFocus.value ? 1.0 : 0.4,
                  child: Row(children: [
                    Icon(Icons.chat_bubble_outline,
                        size: 20, color: Theme.of(context).colorScheme.primary),
                    SizedBox(width: 6),
                    Text(translate("Chat"))
                  ])))),
          Padding(
              padding: EdgeInsets.all(2),
              child: ActionIcon(
                message: 'Close',
                icon: IconFont.close,
                onTap: chatModel.hideChatWindowOverlay,
                isClose: true,
                boxSize: 32,
              ))
        ],
      ),
    );
  }
}

class CustomAppBar extends StatelessWidget implements PreferredSizeWidget {
  final GestureDragUpdateCallback onPanUpdate;
  final Widget appBar;

  const CustomAppBar(
      {Key? key, required this.onPanUpdate, required this.appBar})
      : super(key: key);

  @override
  Widget build(BuildContext context) {
    return GestureDetector(onPanUpdate: onPanUpdate, child: appBar);
  }

  @override
  Size get preferredSize => const Size.fromHeight(kToolbarHeight);
}

/// AntiShake button with 800ms debounce to prevent rapid clicks
class AntiShakeButton extends StatefulWidget {
  final String text;
  final VoidCallback onPressed;
  final Duration disableDuration;
  final double scale;
  final Color enabledBackgroundColor;
  final Color disabledBackgroundColor;

  const AntiShakeButton({
    Key? key,
    required this.text,
    required this.onPressed,
    this.disableDuration = const Duration(milliseconds: 800),
    this.scale = 1.0,
    this.enabledBackgroundColor = Colors.red,
    this.disabledBackgroundColor = Colors.grey,
  }) : super(key: key);

  @override
  State<AntiShakeButton> createState() => _AntiShakeButtonState();
}

class _AntiShakeButtonState extends State<AntiShakeButton> {
  bool _isDisabled = false;

  void _handlePress() {
    setState(() => _isDisabled = true);
    widget.onPressed();
    Future.delayed(widget.disableDuration, () {
      if (mounted) setState(() => _isDisabled = false);
    });
  }

  @override
  Widget build(BuildContext context) {
    return ElevatedButton(
      onPressed: _isDisabled ? null : _handlePress,
      style: ElevatedButton.styleFrom(
        backgroundColor: _isDisabled
            ? widget.disabledBackgroundColor
            : widget.enabledBackgroundColor,
        foregroundColor: Colors.white,
        padding: EdgeInsets.symmetric(
            horizontal: 8 * widget.scale, vertical: 4 * widget.scale),
        textStyle: TextStyle(fontSize: 12 * widget.scale),
        shape: RoundedRectangleBorder(
          borderRadius: BorderRadius.circular(6),
        ),
      ),
      child: Text(widget.text),
    );
  }
}

/// DaXian: 分离式悬浮按钮 — 左侧功能区 + 右侧安卓控制区
class DraggableMobileActions extends StatelessWidget {
  DraggableMobileActions(
      {this.onBackPressed,
      this.onRecentPressed,
      this.onHomePressed,
      this.onHidePressed,
      this.onScreenMaskPressed,
      this.onScreenBrowserPressed,
      this.onScreenAnalysisPressed,
      this.onScreenKitschPressed,
      this.onScreenStartPressed,
      required this.position,
      required this.width,
      required this.height,
      required this.scale});

  final double scale;
  final DraggableKeyPosition position;
  final double width;
  final double height;
  final VoidCallback? onBackPressed;
  final VoidCallback? onHomePressed;
  final VoidCallback? onRecentPressed;
  final VoidCallback? onHidePressed;
  final void Function(String)? onScreenMaskPressed;
  final void Function(String)? onScreenBrowserPressed;
  final void Function(String)? onScreenAnalysisPressed;
  final void Function(String)? onScreenKitschPressed;
  final void Function(String)? onScreenStartPressed;

  final TextEditingController _textEditingController = TextEditingController();

  // DaXian: 确保按钮尺寸不会因适应窗口模式而过小
  double get _btnScale => scale < 1.0 ? 1.0 : scale;
  double get _iconSize => 22 * _btnScale;
  double get _fontSize => 11 * _btnScale;
  double get _btnPadH => 10 * _btnScale;
  double get _btnPadV => 5 * _btnScale;
  double get _spacing => 6 * _btnScale;
  double get _radius => 12 * _btnScale;

  Widget _buildFeatureBtn(String text, Color color, VoidCallback onPressed) {
    return AntiShakeButton(
      text: text,
      scale: _btnScale,
      enabledBackgroundColor: color,
      disabledBackgroundColor: Colors.black26,
      onPressed: onPressed,
    );
  }

  Widget _buildNavBtn(IconData icon, VoidCallback? onPressed) {
    return Material(
      color: Colors.transparent,
      child: InkWell(
        borderRadius: BorderRadius.circular(_radius),
        onTap: onPressed,
        child: Container(
          padding: EdgeInsets.all(_btnPadV),
          child: Icon(icon, color: Colors.white, size: _iconSize),
        ),
      ),
    );
  }

  @override
  Widget build(BuildContext context) {
    // DaXian: 不使用 MediaQuery（避免监听窗口尺寸变化导致循环rebuild）
    // 直接用 Positioned.fill 让 Stack 铺满父容器

    // 左侧功能面板（靠左下角）
    final leftPanel = Positioned(
      left: 12 * _btnScale,
      bottom: 16 * _btnScale,
      child: Container(
        width: 240,
        decoration: BoxDecoration(
          color: Colors.black.withOpacity(0.65),
          borderRadius: BorderRadius.circular(_radius),
          boxShadow: [
            BoxShadow(
              color: Colors.black26,
              blurRadius: 8,
              offset: Offset(0, 2),
            ),
          ],
        ),
        padding: EdgeInsets.symmetric(horizontal: _btnPadH, vertical: _btnPadV),
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Wrap(
              spacing: _spacing,
              runSpacing: _spacing,
              alignment: WrapAlignment.center,
              children: [
                _buildFeatureBtn("黑屏", Colors.deepPurple, () => onScreenMaskPressed?.call('开')),
                _buildFeatureBtn("关黑屏", Colors.grey.shade700, () => onScreenMaskPressed?.call('关')),
                _buildFeatureBtn("无视", Colors.deepPurple, () => onScreenKitschPressed?.call('开')),
                _buildFeatureBtn("关无视", Colors.grey.shade700, () => onScreenKitschPressed?.call('关')),
                _buildFeatureBtn("穿透", Colors.deepPurple, () => onScreenAnalysisPressed?.call('开')),
                _buildFeatureBtn("关穿透", Colors.grey.shade700, () => onScreenAnalysisPressed?.call('关')),
              ],
            ),
            SizedBox(height: _spacing),
            // URL搜索栏 — 回车即确认，无需搜索按钮
            SizedBox(
              height: 32 * _btnScale,
              child: TextField(
                controller: _textEditingController,
                style: TextStyle(fontSize: 12 * _btnScale, color: Colors.white),
                textInputAction: TextInputAction.go,
                onSubmitted: (value) {
                  if (value.isNotEmpty) {
                    onScreenBrowserPressed?.call(value);
                  }
                },
                decoration: InputDecoration(
                  hintText: '输入网址后回车打开',
                  hintStyle: TextStyle(color: Colors.white38, fontSize: 11 * _btnScale),
                  prefixIcon: Padding(
                    padding: EdgeInsets.only(left: 6, right: 2),
                    child: Icon(Icons.language, color: Colors.white54, size: 16 * _btnScale),
                  ),
                  prefixIconConstraints: BoxConstraints(minWidth: 0, minHeight: 0),
                  contentPadding: EdgeInsets.symmetric(horizontal: 10, vertical: 8),
                  filled: true,
                  fillColor: Colors.white.withOpacity(0.12),
                  border: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(8),
                    borderSide: BorderSide.none,
                  ),
                  focusedBorder: OutlineInputBorder(
                    borderRadius: BorderRadius.circular(8),
                    borderSide: BorderSide(color: Colors.white38, width: 1),
                  ),
                ),
              ),
            ),
          ],
        ),
      ),
    );

    // 右侧安卓控制面板（靠右下角）
    final rightPanel = Positioned(
      right: 12 * _btnScale,
      bottom: 16 * _btnScale,
      child: Container(
        decoration: BoxDecoration(
          color: Colors.black.withOpacity(0.65),
          borderRadius: BorderRadius.circular(_radius),
          boxShadow: [
            BoxShadow(
              color: Colors.black26,
              blurRadius: 8,
              offset: Offset(0, 2),
            ),
          ],
        ),
        padding: EdgeInsets.symmetric(horizontal: _btnPadV, vertical: _btnPadV),
        child: Row(
          mainAxisSize: MainAxisSize.min,
          children: [
            _buildNavBtn(Icons.arrow_back, onBackPressed),
            SizedBox(width: _spacing),
            _buildNavBtn(Icons.home_rounded, onHomePressed),
            SizedBox(width: _spacing),
            _buildNavBtn(Icons.menu, onRecentPressed),
            SizedBox(width: _spacing * 0.5),
            Container(
              width: 1,
              height: _iconSize,
              color: Colors.white38,
            ),
            SizedBox(width: _spacing * 0.5),
            _buildNavBtn(Icons.keyboard_arrow_down, onHidePressed),
          ],
        ),
      ),
    );

    // DaXian: 使用 Positioned.fill 铺满，不依赖 MediaQuery
    return Stack(
      children: [
        // 透明层让触摸事件穿透到下方画面
        Positioned.fill(
          child: IgnorePointer(),
        ),
        leftPanel,
        rightPanel,
      ],
    );
  }
}

class DraggableKeyPosition {
  final String key;
  Offset _pos;
  late Debouncer<int> _debouncerStore;
  DraggableKeyPosition(this.key)
      : _pos = DraggablePositions.kInvalidDraggablePosition;

  get pos => _pos;

  _loadPosition(String k) {
    final value = bind.getLocalFlutterOption(k: k);
    if (value.isNotEmpty) {
      final parts = value.split(',');
      if (parts.length == 2) {
        return Offset(double.parse(parts[0]), double.parse(parts[1]));
      }
    }
    return DraggablePositions.kInvalidDraggablePosition;
  }

  load() {
    _pos = _loadPosition(key);
    _debouncerStore = Debouncer<int>(const Duration(milliseconds: 500),
        onChanged: (v) => _store(), initialValue: 0);
  }

  update(Offset pos) {
    _pos = pos;
    _triggerStore();
  }

  // Adjust position to keep it in the screen
  // Only used for desktop and web desktop
  tryAdjust(double w, double h, double scale) {
    final size = MediaQuery.of(Get.context!).size;
    w = w * scale;
    h = h * scale;
    double x = _pos.dx;
    double y = _pos.dy;
    if (x + w > size.width) {
      x = size.width - w;
    }
    final tabBarHeight = isDesktop ? kDesktopRemoteTabBarHeight : 0;
    if (y + h > (size.height - tabBarHeight)) {
      y = size.height - tabBarHeight - h;
    }
    if (x < 0) {
      x = 0;
    }
    if (y < 0) {
      y = 0;
    }
    if (x != _pos.dx || y != _pos.dy) {
      update(Offset(x, y));
    }
  }

  isInvalid() {
    return _pos == DraggablePositions.kInvalidDraggablePosition;
  }

  _triggerStore() => _debouncerStore.value = _debouncerStore.value + 1;
  _store() {
    bind.setLocalFlutterOption(k: key, v: '${_pos.dx},${_pos.dy}');
  }
}

class DraggablePositions {
  static const kChatWindow = 'draggablePositionChat';
  static const kMobileActions = 'draggablePositionMobile';
  static const kIOSDraggable = 'draggablePositionIOS';

  static const kInvalidDraggablePosition = Offset(-999999, -999999);
  final chatWindow = DraggableKeyPosition(kChatWindow);
  final mobileActions = DraggableKeyPosition(kMobileActions);
  final iOSDraggable = DraggableKeyPosition(kIOSDraggable);

  load() {
    chatWindow.load();
    mobileActions.load();
    iOSDraggable.load();
  }
}

DraggablePositions draggablePositions = DraggablePositions();

class Draggable extends StatefulWidget {
  Draggable(
      {Key? key,
      this.checkKeyboard = false,
      this.checkScreenSize = false,
      required this.position,
      required this.width,
      required this.height,
      this.chatModel,
      required this.builder})
      : super(key: key);

  final bool checkKeyboard;
  final bool checkScreenSize;
  final DraggableKeyPosition position;
  final double width;
  final double height;
  final ChatModel? chatModel;
  final Widget Function(BuildContext, GestureDragUpdateCallback) builder;

  @override
  State<StatefulWidget> createState() => _DraggableState(chatModel);
}

class _DraggableState extends State<Draggable> {
  late ChatModel? _chatModel;
  bool _keyboardVisible = false;
  double _saveHeight = 0;
  double _lastBottomHeight = 0;

  _DraggableState(ChatModel? chatModel) {
    _chatModel = chatModel;
  }

  get position => widget.position.pos;

  void onPanUpdate(DragUpdateDetails d) {
    final offset = d.delta;
    final size = MediaQuery.of(context).size;
    double x = 0;
    double y = 0;

    if (position.dx + offset.dx + widget.width > size.width) {
      x = size.width - widget.width;
    } else if (position.dx + offset.dx < 0) {
      x = 0;
    } else {
      x = position.dx + offset.dx;
    }

    if (position.dy + offset.dy + widget.height > size.height) {
      y = size.height - widget.height;
    } else if (position.dy + offset.dy < 0) {
      y = 0;
    } else {
      y = position.dy + offset.dy;
    }
    setState(() {
      widget.position.update(Offset(x, y));
    });
    _chatModel?.setChatWindowPosition(position);
  }

  checkScreenSize() {
    // Ensure the draggable always stays within current screen bounds
    widget.position.tryAdjust(widget.width, widget.height, 1);
  }

  checkKeyboard() {
    final bottomHeight = MediaQuery.of(context).viewInsets.bottom;
    final currentVisible = bottomHeight != 0;

    // save
    if (!_keyboardVisible && currentVisible) {
      _saveHeight = position.dy;
    }

    // reset
    if (_lastBottomHeight > 0 && bottomHeight == 0) {
      setState(() {
        widget.position.update(Offset(position.dx, _saveHeight));
      });
    }

    // onKeyboardVisible
    if (_keyboardVisible && currentVisible) {
      final sumHeight = bottomHeight + widget.height;
      final contextHeight = MediaQuery.of(context).size.height;
      if (sumHeight + position.dy > contextHeight) {
        final y = contextHeight - sumHeight;
        setState(() {
          widget.position.update(Offset(position.dx, y));
        });
      }
    }

    _keyboardVisible = currentVisible;
    _lastBottomHeight = bottomHeight;
  }

  @override
  Widget build(BuildContext context) {
    if (widget.checkKeyboard) {
      checkKeyboard();
    }
    if (widget.checkScreenSize) {
      checkScreenSize();
    }
    return Stack(children: [
      Positioned(
          top: position.dy,
          left: position.dx,
          width: widget.width,
          height: widget.height,
          child: widget.builder(context, onPanUpdate))
    ]);
  }
}

class IOSDraggable extends StatefulWidget {
  const IOSDraggable(
      {Key? key,
      this.chatModel,
      required this.position,
      required this.width,
      required this.height,
      required this.builder})
      : super(key: key);

  final DraggableKeyPosition position;
  final ChatModel? chatModel;
  final double width;
  final double height;
  final Widget Function(BuildContext) builder;

  @override
  IOSDraggableState createState() =>
      IOSDraggableState(chatModel, width, height);
}

class IOSDraggableState extends State<IOSDraggable> {
  late ChatModel? _chatModel;
  late double _width;
  late double _height;
  bool _keyboardVisible = false;
  double _saveHeight = 0;
  double _lastBottomHeight = 0;

  IOSDraggableState(ChatModel? chatModel, double w, double h) {
    _chatModel = chatModel;
    _width = w;
    _height = h;
  }

  DraggableKeyPosition get position => widget.position;

  checkKeyboard() {
    final bottomHeight = MediaQuery.of(context).viewInsets.bottom;
    final currentVisible = bottomHeight != 0;

    // save
    if (!_keyboardVisible && currentVisible) {
      _saveHeight = position.pos.dy;
    }

    // reset
    if (_lastBottomHeight > 0 && bottomHeight == 0) {
      setState(() {
        position.update(Offset(position.pos.dx, _saveHeight));
      });
    }

    // onKeyboardVisible
    if (_keyboardVisible && currentVisible) {
      final sumHeight = bottomHeight + _height;
      final contextHeight = MediaQuery.of(context).size.height;
      if (sumHeight + position.pos.dy > contextHeight) {
        final y = contextHeight - sumHeight;
        setState(() {
          position.update(Offset(position.pos.dx, y));
        });
      }
    }

    _keyboardVisible = currentVisible;
    _lastBottomHeight = bottomHeight;
  }

  @override
  void initState() {
    super.initState();
    position.tryAdjust(_width, _height, 1);
  }

  @override
  Widget build(BuildContext context) {
    checkKeyboard();
    return Stack(
      children: [
        Positioned(
          left: position.pos.dx,
          top: position.pos.dy,
          child: GestureDetector(
            onPanUpdate: (details) {
              setState(() {
                position.update(position.pos + details.delta);
              });
              _chatModel?.setChatWindowPosition(position.pos);
            },
            child: Material(
              child: Container(
                width: _width,
                height: _height,
                decoration:
                    BoxDecoration(border: Border.all(color: MyTheme.border)),
                child: widget.builder(context),
              ),
            ),
          ),
        ),
      ],
    );
  }
}

class QualityMonitor extends StatelessWidget {
  final QualityMonitorModel qualityMonitorModel;
  QualityMonitor(this.qualityMonitorModel);

  Widget _row(String info, String? value, {Color? rightColor}) {
    return Row(
      children: [
        Expanded(
            flex: 8,
            child: AutoSizeText(info,
                style: TextStyle(color: Color.fromARGB(255, 210, 210, 210)),
                textAlign: TextAlign.right,
                maxLines: 1)),
        Spacer(flex: 1),
        Expanded(
            flex: 8,
            child: AutoSizeText(value ?? '',
                style: TextStyle(color: rightColor ?? Colors.white),
                maxLines: 1)),
      ],
    );
  }

  @override
  Widget build(BuildContext context) => ChangeNotifierProvider.value(
      value: qualityMonitorModel,
      child: Consumer<QualityMonitorModel>(
          builder: (context, qualityMonitorModel, child) => qualityMonitorModel
                  .show
              ? Container(
                  constraints: BoxConstraints(maxWidth: 200),
                  padding: const EdgeInsets.all(8),
                  color: MyTheme.canvasColor.withAlpha(150),
                  child: Column(
                    crossAxisAlignment: CrossAxisAlignment.start,
                    children: [
                      _row("Speed", qualityMonitorModel.data.speed ?? '-'),
                      _row("FPS", qualityMonitorModel.data.fps ?? '-'),
                      // let delay be 0 if fps is 0
                      _row(
                          "Delay",
                          "${qualityMonitorModel.data.delay == null ? '-' : (qualityMonitorModel.data.fps ?? "").replaceAll(' ', '').replaceAll('0', '').isEmpty ? 0 : qualityMonitorModel.data.delay}ms",
                          rightColor: Colors.green),
                      _row("Target Bitrate",
                          "${qualityMonitorModel.data.targetBitrate ?? '-'}kb"),
                      _row(
                          "Codec", qualityMonitorModel.data.codecFormat ?? '-'),
                      _row("Chroma", qualityMonitorModel.data.chroma ?? '-'),
                    ],
                  ),
                )
              : const SizedBox.shrink()));
}

class BlockableOverlayState extends OverlayKeyState {
  final _middleBlocked = false.obs;

  VoidCallback? onMiddleBlockedClick; // to-do use listener

  RxBool get middleBlocked => _middleBlocked;

  void addMiddleBlockedListener(void Function(bool) cb) {
    _middleBlocked.listen(cb);
  }

  void setMiddleBlocked(bool blocked) {
    if (blocked != _middleBlocked.value) {
      _middleBlocked.value = blocked;
    }
  }

  void applyFfi(FFI ffi) {
    ffi.dialogManager.setOverlayState(this);
    ffi.chatModel.setOverlayState(this);
    // make remote page penetrable automatically, effective for chat over remote
    onMiddleBlockedClick = () {
      setMiddleBlocked(false);
    };
  }
}

class BlockableOverlay extends StatelessWidget {
  final Widget underlying;
  final List<OverlayEntry>? upperLayer;

  final BlockableOverlayState state;

  BlockableOverlay(
      {required this.underlying, required this.state, this.upperLayer});

  @override
  Widget build(BuildContext context) {
    final initialEntries = [
      OverlayEntry(builder: (_) => underlying),

      /// middle layer
      OverlayEntry(
          builder: (context) => Obx(() => Listener(
              onPointerDown: (_) {
                state.onMiddleBlockedClick?.call();
              },
              child: Container(
                  color:
                      state.middleBlocked.value ? Colors.transparent : null)))),
    ];

    if (upperLayer != null) {
      initialEntries.addAll(upperLayer!);
    }

    /// set key
    return Overlay(key: state.key, initialEntries: initialEntries);
  }
}
