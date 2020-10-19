import Layout from "../components/layout/layout.js";

export default function TopAssets() {

  return (
    <Layout>
      <div className="box">
        <div className="box-top-area">
          <div>
            <div className="box-title">
              <h1>Top Assets</h1>
            </div>
            <div className="box-btn">
              <button>
                <img src="/assets/icons/refresh.svg" alt="refresh now" />
                refresh now
              </button>
            </div>
          </div>
        </div>
        <div className="box-main-area">
          <div className="box-container-table">
          <div className="box-coming-son">
          <img src="/assets/giff/globe_outline.gif" alt="refresh now" />
          <p>
            Coming Son
          </p>
          </div>
          </div>
        </div>
      </div>
    </Layout>
  );
}



