import React, { useState, useCallback, useEffect, useRef } from "react";
import * as ReactDOM from "react-dom/client";

import init, {
  luma_image_data,
  read_qrcodes_from_image_data,
} from "quircs-wasm";

async function main() {
  let stream: MediaStream = null;

  stream = await navigator.mediaDevices.getUserMedia({ video: true });
  /* use the stream */
}

async function get_video_devices() {
  let devices = await navigator.mediaDevices.enumerateDevices();
  return devices.filter(({ kind }) => kind == "videoinput");
}

function QR_CodeReader() {
  const videoElement = useRef<HTMLVideoElement>();
  const canvasElement = useRef<HTMLCanvasElement>();
  const canvasElement2 = useRef<HTMLCanvasElement>();
  const canvasElement3 = useRef<HTMLCanvasElement>();
  const [devices, set_devices] = useState<MediaDeviceInfo[]>([]);

  useEffect(() => {
    let update = (navigator.mediaDevices.ondevicechange = () => {
      console.log("devices changed");
      get_video_devices().then(set_devices);
    });
    update();
    return () => {
      navigator.mediaDevices.ondevicechange = null;
    };
  }, []);

  const [active_deviceId, set_active_deviceId] = useState<string | null>(null);

  const onSelect = useCallback(
    (ev) => {
      const deviceId = ev.target.value;
      console.log("Set active device:", deviceId);
      set_active_deviceId(deviceId !== "null" ? deviceId : null);
    },
    [devices]
  );

  const active_stream = useRef<MediaStream | null>(null);
  const [stream_dimensions, set_stream_dimensions] = useState<{
    height: number;
    width: number;
  }>({ height: 300, width: 300 });

  useEffect(() => {
    console.log("Switch video to device:", active_deviceId);
    if (active_deviceId) {
      navigator.mediaDevices
        .getUserMedia({ video: { deviceId: { exact: active_deviceId } } })
        .then((stream) => {
          active_stream.current = stream;
          if (videoElement.current) {
            videoElement.current.autoplay = true;
            videoElement.current.srcObject = stream;
            const { height, width } = stream.getVideoTracks()[0].getSettings();
            set_stream_dimensions({ height, width });
          }
        })
        .catch((err) => {
          console.log(err);
          active_stream.current = null;
        });
    }

    return () => {
      console.log("close stream callback if active");

      if (active_stream.current) {
        console.log("close stream callback");
        videoElement.current.srcObject = null;
        for (let track of active_stream.current.getTracks()) {
          track.stop();
        }
      }
    };
  }, [active_deviceId]);

  const [qr_codes, set_qr_codes] = useState<string[]>([]);

  useEffect(() => {
    const scan = () => {
      const canvas = canvasElement.current;
      const video = videoElement.current;

      var context = canvas.getContext("2d", {willReadFrequently: true, alpha: false, desynchronized: true});
      context.drawImage(video, 0, 0, canvas.width, canvas.height);
      const img_data = context.getImageData(0, 0, canvas.width, canvas.height);
      if (videoElement.current.srcObject) {
        const res = read_qrcodes_from_image_data(img_data, true);
        console.log({ res });
        var context = canvasElement2.current.getContext("2d");
        context.clearRect(0, 0, canvas.width, canvas.height);
        let i = 0,
          codes = [];
        for (let qr of res) {
          context.lineWidth = 10;
          if (qr.data["content"]) {
            context.strokeStyle = "green";
          } else {
            context.strokeStyle = "red";
          }

          context.fillStyle = "grey";
          context.beginPath();
          context.moveTo(qr.corners[0].x, qr.corners[0].y);
          context.lineTo(qr.corners[1].x, qr.corners[1].y);
          context.lineTo(qr.corners[2].x, qr.corners[2].y);
          context.lineTo(qr.corners[3].x, qr.corners[3].y);
          context.fill();

          context.beginPath();
          context.moveTo(qr.corners[0].x, qr.corners[0].y);
          context.lineTo(qr.corners[1].x, qr.corners[1].y);
          context.lineTo(qr.corners[2].x, qr.corners[2].y);
          context.lineTo(qr.corners[3].x, qr.corners[3].y);
          context.lineTo(qr.corners[0].x, qr.corners[0].y);
          context.lineTo(qr.corners[1].x, qr.corners[1].y);
          context.stroke();

          context.font = "40px Arial";
          context.fillStyle = "white";
          context.fillText(
            String(i),
            qr.corners[0].x + 5,
            qr.corners[0].y + 35
          );
          context.font = "20px Arial";
          context.fillStyle = "white";
          const x = qr.corners[0].x + 5;
          const y = qr.corners[0].y + 45;
          if (qr.data["error"]) {
            const error = qr.data["error"];
            console.log(error);
            context.fillStyle = "orange";
            context.fillText(error, x, y + 20);
            codes.push(`${i}: [Error]: ${error}`);
          } else {
            const data = qr.data["content"];
            context.fillText(`version: ${data.version}`, x, y + 20);
            context.fillText(`mask: ${data.mask}`, x, y + 20 * 2);
            context.fillText(`ecc_level: ${data.ecc_level}`, x, y + 20 * 3);
            context.fillText(`data_type: ${data.data_type}`, x, y + 20 * 4);
            let payload = data.payload;
            codes.push(`${i}: ${String.fromCharCode.apply(null, payload)}`);
          }

          i++;
        }

        set_qr_codes(codes);

        var context = canvasElement3.current.getContext("2d");
        context.putImageData(luma_image_data(img_data, true, false), 0, 0);
      }
    };

    setInterval(scan, 250);
  }, []);

  return (
    <div>
      <select
        onChange={onSelect}
        name="video device"
        id="video-device-selection"
      >
        <option value={"null"}>None</option>
        {devices.map((device) => {
          return (
            <option value={device.deviceId} key={device.deviceId}>
              {device.label}
            </option>
          );
        })}
      </select>
      <div>
        {qr_codes.map((code) => {
          return <p>{code}</p>;
        })}
      </div>
      <div style={{ position: "relative", height: "480px" }}>
        <video ref={videoElement} style={{ position: "absolute" }}></video>
        <canvas
          ref={canvasElement2}
          width={stream_dimensions.width}
          height={stream_dimensions.height}
          style={{ position: "absolute", top: 0, opacity: 0.87 }}
        ></canvas>
      </div>
      <canvas
        ref={canvasElement}
        width={stream_dimensions.width}
        height={stream_dimensions.height}
        style={{ display: "none" }}
      ></canvas>
      <canvas
        ref={canvasElement3}
        width={stream_dimensions.width}
        height={stream_dimensions.height}
      ></canvas>
    </div>
  );
}

window.onload = async () => {
  await init("./quircs_wasm_bg.wasm");

  const domContainer = document.querySelector("#app");
  const root = ReactDOM.createRoot(domContainer);
  root.render(<QR_CodeReader />);
};
