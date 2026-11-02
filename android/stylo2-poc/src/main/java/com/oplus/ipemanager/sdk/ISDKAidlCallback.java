package com.oplus.ipemanager.sdk;

import android.os.Binder;
import android.os.IBinder;
import android.os.IInterface;
import android.os.Parcel;
import android.os.RemoteException;

/** AIDL-style callback interface (cleaned) */
public interface ISDKAidlCallback extends IInterface {
    String DESCRIPTOR = "com.oplus.ipemanager.sdk.ISDKAidlCallback";

    void onConnectionChanged(int state) throws RemoteException;
    void onDemoModeEnableChange(boolean enabled) throws RemoteException;
    void onFunctionFeedbackStateChange(boolean enabled) throws RemoteException;
    void onVibrationSwitchStateChange(boolean enabled) throws RemoteException;

    abstract class Stub extends Binder implements ISDKAidlCallback {
        static final int TRANSACTION_onConnectionChanged = 1;
        static final int TRANSACTION_onVibrationSwitchStateChange = 2;
        static final int TRANSACTION_onFunctionFeedbackStateChange = 3;
        static final int TRANSACTION_onDemoModeEnableChange = 4;

        public Stub() {
            attachInterface(this, DESCRIPTOR);
        }

        public static ISDKAidlCallback asInterface(IBinder obj) {
            if (obj == null) return null;
            IInterface iin = obj.queryLocalInterface(DESCRIPTOR);
            return (iin instanceof ISDKAidlCallback) ? (ISDKAidlCallback) iin : new Proxy(obj);
        }

        @Override public IBinder asBinder() { return this; }

        @Override
        public boolean onTransact(int code, Parcel data, Parcel reply, int flags) throws RemoteException {
            if (code == INTERFACE_TRANSACTION) {
                reply.writeString(DESCRIPTOR);
                return true;
            }
            data.enforceInterface(DESCRIPTOR);
            switch (code) {
                case TRANSACTION_onConnectionChanged: {
                    int state = data.readInt();
                    this.onConnectionChanged(state);
                    return true;
                }
                case TRANSACTION_onVibrationSwitchStateChange: {
                    boolean enabled = data.readInt() != 0;
                    this.onVibrationSwitchStateChange(enabled);
                    return true;
                }
                case TRANSACTION_onFunctionFeedbackStateChange: {
                    boolean enabled = data.readInt() != 0;
                    this.onFunctionFeedbackStateChange(enabled);
                    return true;
                }
                case TRANSACTION_onDemoModeEnableChange: {
                    boolean enabled = data.readInt() != 0;
                    this.onDemoModeEnableChange(enabled);
                    return true;
                }
            }
            return super.onTransact(code, data, reply, flags);
        }

        private static final class Proxy implements ISDKAidlCallback {
            private final IBinder mRemote;
            Proxy(IBinder remote) { mRemote = remote; }
            @Override public IBinder asBinder() { return mRemote; }
            @Override public void onConnectionChanged(int state) throws RemoteException {
                Parcel _data = Parcel.obtain();
                try {
                    _data.writeInterfaceToken(DESCRIPTOR);
                    _data.writeInt(state);
                    mRemote.transact(TRANSACTION_onConnectionChanged, _data, null, IBinder.FLAG_ONEWAY);
                } finally { _data.recycle(); }
            }
            @Override public void onDemoModeEnableChange(boolean enabled) throws RemoteException {
                Parcel _data = Parcel.obtain();
                try {
                    _data.writeInterfaceToken(DESCRIPTOR);
                    _data.writeInt(enabled ? 1 : 0);
                    mRemote.transact(TRANSACTION_onDemoModeEnableChange, _data, null, IBinder.FLAG_ONEWAY);
                } finally { _data.recycle(); }
            }
            @Override public void onFunctionFeedbackStateChange(boolean enabled) throws RemoteException {
                Parcel _data = Parcel.obtain();
                try {
                    _data.writeInterfaceToken(DESCRIPTOR);
                    _data.writeInt(enabled ? 1 : 0);
                    mRemote.transact(TRANSACTION_onFunctionFeedbackStateChange, _data, null, IBinder.FLAG_ONEWAY);
                } finally { _data.recycle(); }
            }
            @Override public void onVibrationSwitchStateChange(boolean enabled) throws RemoteException {
                Parcel _data = Parcel.obtain();
                try {
                    _data.writeInterfaceToken(DESCRIPTOR);
                    _data.writeInt(enabled ? 1 : 0);
                    mRemote.transact(TRANSACTION_onVibrationSwitchStateChange, _data, null, IBinder.FLAG_ONEWAY);
                } finally { _data.recycle(); }
            }
        }
    }
}
