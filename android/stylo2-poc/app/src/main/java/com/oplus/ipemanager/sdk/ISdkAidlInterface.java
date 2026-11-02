package com.oplus.ipemanager.sdk;

import android.os.Binder;
import android.os.IBinder;
import android.os.IInterface;
import android.os.Parcel;
import android.os.RemoteException;
import java.util.List;

/** Main SDK AIDL interface (cleaned) */
public interface ISdkAidlInterface extends IInterface {
    String DESCRIPTOR = "com.oplus.ipemanager.sdk.ISdkAidlInterface";

    // Query
    int getPencilConnectState() throws RemoteException;
    int getSuportSdkVersion() throws RemoteException;
    boolean getVibrationSwitchState() throws RemoteException;
    boolean isDemoModeEnable() throws RemoteException;
    boolean isFunctionVibrationEnable() throws RemoteException;
    List<String> getSupportFeatureList() throws RemoteException; // optional helper

    // Control
    void startVibration(int type) throws RemoteException;
    void setVibrationType(int type) throws RemoteException;
    void startFeedBackVibration() throws RemoteException;
    void stopFeedBackVibration() throws RemoteException;
    void enableDemoMode(boolean enabled) throws RemoteException;

    // Callbacks
    void setISdkAidlCallback(ISDKAidlCallback cb) throws RemoteException;
    void unsetISdkAidlCallback(ISDKAidlCallback cb) throws RemoteException;

    abstract class Stub extends Binder implements ISdkAidlInterface {
        static final int TRANSACTION_getPencilConnectState = 1;
        static final int TRANSACTION_getVibrationSwitchState = 2;
        static final int TRANSACTION_isFunctionVibrationEnable = 3;
        static final int TRANSACTION_isDemoModeEnable = 4;
        static final int TRANSACTION_enableDemoMode = 5;
        static final int TRANSACTION_getSuportSdkVersion = 6;
        static final int TRANSACTION_setVibrationType = 7;
        static final int TRANSACTION_startVibration = 8;
        static final int TRANSACTION_setISdkAidlCallback = 9;
        static final int TRANSACTION_unsetISdkAidlCallback = 10;
        static final int TRANSACTION_startFeedBackVibration = 11;
        static final int TRANSACTION_stopFeedBackVibration = 12;

        public Stub() {
            attachInterface(this, DESCRIPTOR);
        }

        public static ISdkAidlInterface asInterface(IBinder obj) {
            if (obj == null) return null;
            IInterface iin = obj.queryLocalInterface(DESCRIPTOR);
            return (iin instanceof ISdkAidlInterface) ? (ISdkAidlInterface) iin : new Proxy(obj);
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
                case TRANSACTION_getPencilConnectState:
                    reply.writeNoException();
                    reply.writeInt(getPencilConnectState());
                    return true;
                case TRANSACTION_getVibrationSwitchState:
                    reply.writeNoException();
                    reply.writeInt(getVibrationSwitchState() ? 1 : 0);
                    return true;
                case TRANSACTION_isFunctionVibrationEnable:
                    reply.writeNoException();
                    reply.writeInt(isFunctionVibrationEnable() ? 1 : 0);
                    return true;
                case TRANSACTION_isDemoModeEnable:
                    reply.writeNoException();
                    reply.writeInt(isDemoModeEnable() ? 1 : 0);
                    return true;
                case TRANSACTION_enableDemoMode:
                    enableDemoMode(data.readInt() != 0);
                    reply.writeNoException();
                    return true;
                case TRANSACTION_getSuportSdkVersion:
                    reply.writeNoException();
                    reply.writeInt(getSuportSdkVersion());
                    return true;
                case TRANSACTION_setVibrationType:
                    setVibrationType(data.readInt());
                    reply.writeNoException();
                    return true;
                case TRANSACTION_startVibration:
                    startVibration(data.readInt());
                    reply.writeNoException();
                    return true;
                case TRANSACTION_setISdkAidlCallback: {
                    IBinder b = data.readStrongBinder();
                    ISDKAidlCallback cb = ISDKAidlCallback.Stub.asInterface(b);
                    setISdkAidlCallback(cb);
                    reply.writeNoException();
                    return true;
                }
                case TRANSACTION_unsetISdkAidlCallback: {
                    IBinder b = data.readStrongBinder();
                    ISDKAidlCallback cb = ISDKAidlCallback.Stub.asInterface(b);
                    unsetISdkAidlCallback(cb);
                    reply.writeNoException();
                    return true;
                }
                case TRANSACTION_startFeedBackVibration:
                    startFeedBackVibration();
                    reply.writeNoException();
                    return true;
                case TRANSACTION_stopFeedBackVibration:
                    stopFeedBackVibration();
                    reply.writeNoException();
                    return true;
            }
            return super.onTransact(code, data, reply, flags);
        }

        private static final class Proxy implements ISdkAidlInterface {
            private final IBinder mRemote;
            Proxy(IBinder remote) { mRemote = remote; }
            @Override public IBinder asBinder() { return mRemote; }

            @Override public int getPencilConnectState() throws RemoteException {
                Parcel _d = Parcel.obtain(), _r = Parcel.obtain();
                try {
                    _d.writeInterfaceToken(DESCRIPTOR);
                    mRemote.transact(TRANSACTION_getPencilConnectState, _d, _r, 0);
                    _r.readException();
                    return _r.readInt();
                } finally { _r.recycle(); _d.recycle(); }
            }

            @Override public int getSuportSdkVersion() throws RemoteException {
                Parcel _d = Parcel.obtain(), _r = Parcel.obtain();
                try {
                    _d.writeInterfaceToken(DESCRIPTOR);
                    mRemote.transact(TRANSACTION_getSuportSdkVersion, _d, _r, 0);
                    _r.readException();
                    return _r.readInt();
                } finally { _r.recycle(); _d.recycle(); }
            }

            @Override public boolean getVibrationSwitchState() throws RemoteException {
                Parcel _d = Parcel.obtain(), _r = Parcel.obtain();
                try {
                    _d.writeInterfaceToken(DESCRIPTOR);
                    mRemote.transact(TRANSACTION_getVibrationSwitchState, _d, _r, 0);
                    _r.readException();
                    return _r.readInt() != 0;
                } finally { _r.recycle(); _d.recycle(); }
            }

            @Override public boolean isDemoModeEnable() throws RemoteException {
                Parcel _d = Parcel.obtain(), _r = Parcel.obtain();
                try {
                    _d.writeInterfaceToken(DESCRIPTOR);
                    mRemote.transact(TRANSACTION_isDemoModeEnable, _d, _r, 0);
                    _r.readException();
                    return _r.readInt() != 0;
                } finally { _r.recycle(); _d.recycle(); }
            }

            @Override public boolean isFunctionVibrationEnable() throws RemoteException {
                Parcel _d = Parcel.obtain(), _r = Parcel.obtain();
                try {
                    _d.writeInterfaceToken(DESCRIPTOR);
                    mRemote.transact(TRANSACTION_isFunctionVibrationEnable, _d, _r, 0);
                    _r.readException();
                    return _r.readInt() != 0;
                } finally { _r.recycle(); _d.recycle(); }
            }

            @Override public List<String> getSupportFeatureList() throws RemoteException {
                // Not strictly required for your current uses; implement if needed.
                throw new RemoteException("getSupportFeatureList not implemented in proxy.");
            }

            @Override public void startVibration(int type) throws RemoteException {
                Parcel _d = Parcel.obtain(), _r = Parcel.obtain();
                try {
                    _d.writeInterfaceToken(DESCRIPTOR);
                    _d.writeInt(type);
                    mRemote.transact(TRANSACTION_startVibration, _d, _r, 0);
                    _r.readException();
                } finally { _r.recycle(); _d.recycle(); }
            }

            @Override public void setVibrationType(int type) throws RemoteException {
                Parcel _d = Parcel.obtain(), _r = Parcel.obtain();
                try {
                    _d.writeInterfaceToken(DESCRIPTOR);
                    _d.writeInt(type);
                    mRemote.transact(TRANSACTION_setVibrationType, _d, _r, 0);
                    _r.readException();
                } finally { _r.recycle(); _d.recycle(); }
            }

            @Override public void startFeedBackVibration() throws RemoteException {
                Parcel _d = Parcel.obtain(), _r = Parcel.obtain();
                try {
                    _d.writeInterfaceToken(DESCRIPTOR);
                    mRemote.transact(TRANSACTION_startFeedBackVibration, _d, _r, 0);
                    _r.readException();
                } finally { _r.recycle(); _d.recycle(); }
            }

            @Override public void stopFeedBackVibration() throws RemoteException {
                Parcel _d = Parcel.obtain(), _r = Parcel.obtain();
                try {
                    _d.writeInterfaceToken(DESCRIPTOR);
                    mRemote.transact(TRANSACTION_stopFeedBackVibration, _d, _r, 0);
                    _r.readException();
                } finally { _r.recycle(); _d.recycle(); }
            }

            @Override public void enableDemoMode(boolean enabled) throws RemoteException {
                Parcel _d = Parcel.obtain(), _r = Parcel.obtain();
                try {
                    _d.writeInterfaceToken(DESCRIPTOR);
                    _d.writeInt(enabled ? 1 : 0);
                    mRemote.transact(TRANSACTION_enableDemoMode, _d, _r, 0);
                    _r.readException();
                } finally { _r.recycle(); _d.recycle(); }
            }

            @Override public void setISdkAidlCallback(ISDKAidlCallback cb) throws RemoteException {
                Parcel _d = Parcel.obtain(), _r = Parcel.obtain();
                try {
                    _d.writeInterfaceToken(DESCRIPTOR);
                    _d.writeStrongBinder(cb != null ? cb.asBinder() : null);
                    mRemote.transact(TRANSACTION_setISdkAidlCallback, _d, _r, 0);
                    _r.readException();
                } finally { _r.recycle(); _d.recycle(); }
            }

            @Override public void unsetISdkAidlCallback(ISDKAidlCallback cb) throws RemoteException {
                Parcel _d = Parcel.obtain(), _r = Parcel.obtain();
                try {
                    _d.writeInterfaceToken(DESCRIPTOR);
                    _d.writeStrongBinder(cb != null ? cb.asBinder() : null);
                    mRemote.transact(TRANSACTION_unsetISdkAidlCallback, _d, _r, 0);
                    _r.readException();
                } finally { _r.recycle(); _d.recycle(); }
            }
        }
    }
}
