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
            {/* <Table columns={columns} data={dataTable} rowKey={(e) => e.id} /> */}
          </div>
        </div>
      </div>
    </Layout>
  );
}



