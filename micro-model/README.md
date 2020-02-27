# micro-model

Bincode-compatible model crates, used for communication
between the micro-stack services.  These model definitions
can be serialized and deserialized efficiently, but they
must not be changed in one service without being changed
in all other services, since bincode can't support backwards-
compatible changes to data.  [See the Bincode 1.0.0 release
article for more info](http://tyoverby.com/posts/bincode_release.html)
