import 'package:flutter/material.dart';
import 'package:tms/network/security.dart';
import 'package:tms/schema/tms_schema.dart';
import 'package:web_socket_channel/web_socket_channel.dart';

enum NetworkWebSocketState { disconnected, connected }

class NetworkWebSocket {
  // final Future<SharedPreferences> _localStorage = SharedPreferences.getInstance();
  static ValueNotifier<NetworkWebSocketState> wsState = ValueNotifier<NetworkWebSocketState>(NetworkWebSocketState.disconnected);
  late WebSocketChannel _channel;
  final Map<String, List<void Function(SocketMessage message)>> _subscribers = {}; // Topic and function/s

  void setState(NetworkWebSocketState state) => wsState.value = state;
  NetworkWebSocketState getState() => wsState.value;

  Function(SocketMessage) subscribe(String topic, void Function(SocketMessage message) onEvent) {
    if (!_subscribers.containsKey(topic)) {
      _subscribers[topic] = [];
    }

    _subscribers[topic]?.add(onEvent);
    return onEvent;
  }

  void unsubscribe(String topic, Function(SocketMessage) onEvent) {
    if (_subscribers.containsKey(topic)) {
      // Logger().w("Unsubscribing from $topic");
      _subscribers[topic]?.remove(onEvent);
    }
  }

  Future<void> publish(SocketMessage message) async {
    if (getState() == NetworkWebSocketState.connected) {
      try {
        _channel.ready.then((v) {
          NetworkSecurity.encryptMessage(message).then((data) {
            _channel.sink.add(data);
          });
        });
      } catch (e) {
        setState(NetworkWebSocketState.disconnected);
      }
    }
  }

  // This can't return a future
  void _listen() async {
    if (getState() == NetworkWebSocketState.connected) {
      try {
        // Listen to the socket
        _channel.stream.listen((event) {
          // On a regular data message decrypt the message
          NetworkSecurity.decryptMessage(event).then((value) {
            // Convert the message to a socket message type
            SocketMessage m;
            try {
              m = SocketMessage.fromJson(value);
            } catch (e) {
              m = SocketMessage(message: "", subTopic: "", topic: "");
            }
            // Iterate through the subscribers and check if the topic matches
            _subscribers.forEach((topic, functionList) {
              if (topic == m.topic) {
                // If the topic matches run every onEvent function in the list associated with the topic
                for (final onEvent in functionList) {
                  onEvent(m);
                }
              }
            });
          });
        }, onDone: () {
          setState(NetworkWebSocketState.disconnected);
        }, onError: (e) {
          disconnect().then((v) {
            setState(NetworkWebSocketState.disconnected);
          });
        });
      } catch (e) {
        disconnect().then((v) {
          setState(NetworkWebSocketState.disconnected);
        });
      }
    }
  }

  Future<void> connect(String url) async {
    if (getState() != NetworkWebSocketState.connected) {
      try {
        _channel = WebSocketChannel.connect(Uri.parse(url));

        _channel.ready.then((v) {
          setState(NetworkWebSocketState.connected);
          // Logger().i("Connecting using url $url");
          _listen();
        });
      } catch (e) {
        setState(NetworkWebSocketState.disconnected);
      }
    } else {
      try {
        publish(SocketMessage(message: "ping", topic: "none", subTopic: "none"));
      } catch (e) {
        setState(NetworkWebSocketState.disconnected);
      }
    }
  }

  Future<void> disconnect() async {
    if (getState() == NetworkWebSocketState.connected) {
      _channel.sink.close().then((v) {
        setState(NetworkWebSocketState.disconnected);
      });
    }
  }
}
