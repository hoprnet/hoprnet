import React from "react";
import "../../styles/main.scss";

const Modal = ({ modal,hash }) => {
  return (
    <div className={"modal-copy-menu " + [modal ? "show-modal-menu" : ""]}>
      <div className="box-modal-copy">
        <div className="icon-logo">
          <img src="/assets/brand/logo.svg" alt="hopr" />
        </div>
        <div className="content">
          <div>
            <p>{hash}</p>
          </div>
          <h5>copied to clipboard</h5>
          <hr className="hr-alert" />
          <p className="copy-alert">
            this message is only informative it <br />
            closes itself in <span>4 seconds.</span>
          </p>
        </div>
      </div>
    </div>
  );
};

export default Modal;
