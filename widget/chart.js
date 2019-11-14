var contract;
var channelEventRecords = [];
var svg, timeScale, amountScale, countScale, width, height, margin, chartData, focus;
var xAxis, yAxisAmount, yAxisCount, amountLine, countLine;
var dateFormat = d3.timeFormat('%b %d');
var fullMonthDateFormat = d3.timeFormat('%B %d');
var bisectDate = d3.bisector(function(d) { return d.date; }).left;

window.addEventListener('resize', function() {
  initChart();
  renderRecords();
});

window.addEventListener('load', function() {
  initChart();

  if (typeof web3 !== 'undefined') {
    window.web3 = new Web3(web3.currentProvider);
    contract = web3.eth.contract(CONTRACT_ABI).at(CONTRACT_ADDRESS);
    whatchChannel(contract.openedChannel, OPEN_CHANNEL_EVENT);
    whatchChannel(contract.closedChannel, CLOSE_CHANNEL_EVENT);
  } else {
    console.log('No web3 provider');
  }
});

function parseAmount(amount) {
  return amount / 10000;
}

function whatchChannel(event, eventType) {
  var channelEvent = event({}, { fromBlock: 0, toBlock: 'latest' });
  channelEvent.watch(function(error, result) {
    var amount = parseAmount(result.args.amount.c[0]);
    var time = result.args.time.c[0];
    var eventRecord = { type: eventType, amount: amount, time: time };
    channelEventRecords.push(eventRecord);
    renderRecords();
  });
}

function renderRecords() {
  var orderedRecords = channelEventRecords.sort(function(a, b) {
    return a.time - b.time;
  });
  var aggregatedRecords = aggregateRecords(orderedRecords);
  updateChart(aggregatedRecords);
  setOverlayTooltip();
}

function aggregateRecords(records) {
  var channelAmmountBalance = 0;
  var channelStatusCount = 0;
  var aggregatedRecords = [];

  for (var i = 0; i < records.length; i++) {
    var record = records[i];
    var recordEventType = record.type;
    var recordEventAmount = record.amount;

    if (recordEventType === OPEN_CHANNEL_EVENT) {
      channelStatusCount += 1;
      channelAmmountBalance += recordEventAmount;
    } else {
      channelStatusCount -= 1;
      channelAmmountBalance -= recordEventAmount;
    }

    var recordDate = new Date(record.time * 1000);

    aggregatedRecords.push(
      Object.assign({}, records[i], {
        aggCount: channelStatusCount,
        aggAmount: channelAmmountBalance,
        date: recordDate
      })
    );
  }

  return aggregatedRecords;
}

function initChart() {
  margin = { top: 30, right: 70, bottom: 30, left: 60 };
  width = document.getElementById('chart-container').offsetWidth - margin.left - margin.right;
  height = document.getElementById('chart-container').offsetHeight - margin.top - margin.bottom;
  tooltipDimensions = { width: 150, height: 75 };
  timeScale = d3.scaleTime().range([0, width]);
  amountScale = d3.scaleLinear().range([height, 0]);
  countScale = d3.scaleLinear().range([height, 0]);

  d3.selectAll('#chart-container svg').remove();

  svg = d3
    .select('#chart-container')
    .append('svg')
    .attr('width', width + margin.left + margin.right)
    .attr('height', height + margin.top + margin.bottom);

  amountLine = d3
    .line()
    .curve(d3.curveCatmullRom.alpha(0.5))
    .x(function(d) {
      return timeScale(d.date);
    })
    .y(function(d) {
      return amountScale(d.aggAmount);
    });
    

  countLine = d3
    .line()
    .curve(d3.curveCatmullRom.alpha(0.5))
    .x(function(d) {
      return timeScale(d.date);
    })
    .y(function(d) {
      return countScale(d.aggCount);
    });
}

function updateChart(data) {
  console.log("TCL: updateChart -> data", data)
  chartData = data;
  d3.selectAll('.chart-content').remove();

  timeScale.domain(
    d3.extent(data, function(d) {
      return d.date;
    })
  );
  amountScale.domain([
    0,
    d3.max(data, function(d) {
      return Math.max(d.aggAmount);
    })
  ]);
  countScale.domain([
    0,
    d3.max(data, function(d) {
      return Math.max(d.aggCount);
    })
  ]);

  var svgChart = svg
    .append('g')
    .attr('class', 'chart-content')
    .attr('transform', 'translate(' + margin.left + ',' + margin.top + ')');

  svgChart
    .append('path')
    .data([data])
    .attr('class', 'amount-line')
    .style('stroke-width', 1.5)
    .style("stroke-dasharray", ("12, 10"))
    .attr('d', amountLine);

  svgChart
    .append('path')
    .data([data])
    .attr('class', 'count-line')
    .style('stroke-width', 1.5)
    .attr('d', countLine);

  svgChart
    .append('g')
    .attr('class', 'xAxis axis')
    .attr('transform', 'translate(0,' + height + ')')
    .call(
      d3
        .axisBottom(timeScale)
        .tickFormat(dateFormat)
        .ticks(5)
    );

  svgChart
    .append('g')
    .attr('class', 'amountAxis axis')
    .call(d3.axisLeft(amountScale).ticks(5));

  svgChart.append("text")
    .attr("transform", "rotate(-90)")
    .attr("y", 0 - margin.left / 1.5)
    .attr("x",0 - (height / 2))
    .attr("dy", "1em")
    .style("text-anchor", "middle")
    .text("TOTAL STAKED HOPR"); 

  svgChart
    .append('g')
    .attr('class', 'countAxis axis')
    .attr('transform', 'translate(' + width + ' ,0)')
    .call(d3.axisRight(countScale).ticks(5));

  svgChart.append("text")
    .attr("transform", "rotate(-90)")
    .attr("y", width + margin.left / 2)
    .attr("x",0 - (height / 2))
    .attr("dy", "1em")
    .style("text-anchor", "middle")
    .text("# Channles"); 
}

function setOverlayTooltip(){
  d3.selectAll('.focus').remove();
  d3.selectAll('.chart-overlay').remove();

  focus = svg.append('g')
    .attr('class', 'focus')
    .attr('transform', 'translate(' + margin.left + ',' + margin.top + ')')
    .style('display', 'none');

  focus.append('circle')
    .attr('id', 'amount-circle')
    .attr('class', 'tooltip-circle tooltip-circle-amount')
    .attr('r', 5);

  focus.append('circle')
    .attr('id', 'count-circle')
    .attr('class', 'tooltip-circle tooltip-circle-count')
    .attr('r', 5);

  var tooltip = focus.append('g')
  .attr('id', 'tooltip-group')
  
  tooltip.append('rect')
    .attr('class', 'tooltip')
    .attr('width', tooltipDimensions.width)
    .attr('height', tooltipDimensions.height)
    .attr('x', 10)
    .attr('y', -22);

  tooltip.append('text')
    .attr('class', 'tooltip-date')
    .attr('x', 18)
    .attr('y', -2);

  tooltip.append('text')
    .attr('x', 18)
    .attr('y', 18)
    .text('Total Staked:');

  tooltip.append('text')
    .attr('id', 'tooltip-amount')
    .attr('x', 120)
    .attr('y', 18);

  tooltip.append('text')
    .attr('x', 18)
    .attr('y', 38)
    .text('Channels:');

  tooltip.append('text')
    .attr('id', 'tooltip-count')
    .attr('x', 90)
    .attr('y', 38);

  svg.append('rect')
    .attr('class', 'chart-overlay')
    .attr('transform', 'translate(' + margin.left + ',' + margin.top + ')')
    .attr('width', width)
    .attr('height', height)
    .on('mouseover', function() { focus.style('display', null); })
    .on('mouseout', function() { focus.style('display', 'none'); })
    .on('mousemove', mousemove);
}

function mousemove() {
  var x0 = timeScale.invert(d3.mouse(this)[0]),
      i = bisectDate(chartData, x0, 1),
      d0 = chartData[i - 1],
      d1 = chartData[i],
      d = x0 - d0.time > d1.time - x0 ? d1 : d0;
      
  var x = timeScale(d.date);
  var yAmount = amountScale(d.aggAmount);
  var yCount = countScale(d.aggCount);

  focus.select('#amount-circle').attr('transform', `translate(${x},${yAmount})`);
  focus.select('#count-circle').attr('transform', `translate(${x},${yCount})`);

  var y = Math.min(yAmount, yCount);
  var tooltipXOffset = 30;
  var tooltipX = x >  width / 2 
    ? x - tooltipDimensions.width - tooltipXOffset 
    : x + tooltipXOffset; 

  var tooltipYOffset = 10;
  var tooltipY = y > height / 2 
    ? y - tooltipDimensions.height - tooltipYOffset
    : y + tooltipYOffset; 
  focus.select('#tooltip-group').attr('transform', `translate(${tooltipX}, ${tooltipY})`);
  
  focus.select('.tooltip-date').text(fullMonthDateFormat(d.date));
  focus.select('#tooltip-amount').text(d.aggAmount);
  focus.select('#tooltip-count').text(d.aggCount);
}

function zeroFill(number) {
  if (number < 10) {
    return '0' + number;
  } else {
    return String(number);
  }
}

var OPEN_CHANNEL_EVENT = 'openChannel';
var CLOSE_CHANNEL_EVENT = 'closeChannel';
var CONTRACT_ADDRESS = '0x9c9583fdd67648d539da325b3c9d8087d0b6bda2';
var CONTRACT_ABI = [
  {
    constant: false,
    inputs: [
      { internalType: 'address', name: 'partyB', type: 'address' },
      { internalType: 'uint256', name: 'amount', type: 'uint256' }
    ],
    name: 'open',
    outputs: [],
    payable: false,
    stateMutability: 'nonpayable',
    type: 'function'
  },
  {
    constant: true,
    inputs: [],
    name: 'totalAmount',
    outputs: [{ internalType: 'uint256', name: '', type: 'uint256' }],
    payable: false,
    stateMutability: 'view',
    type: 'function'
  },
  {
    constant: false,
    inputs: [{ internalType: 'bytes32', name: 'channelId', type: 'bytes32' }],
    name: 'close',
    outputs: [],
    payable: false,
    stateMutability: 'nonpayable',
    type: 'function'
  },
  {
    constant: true,
    inputs: [],
    name: 'numChannels',
    outputs: [{ internalType: 'uint256', name: '', type: 'uint256' }],
    payable: false,
    stateMutability: 'view',
    type: 'function'
  },
  {
    constant: true,
    inputs: [{ internalType: 'bytes32', name: '', type: 'bytes32' }],
    name: 'Channels',
    outputs: [
      { internalType: 'address', name: 'partyA', type: 'address' },
      { internalType: 'address', name: 'partyB', type: 'address' },
      { internalType: 'uint256', name: 'amount', type: 'uint256' },
      { internalType: 'bool', name: 'isOpen', type: 'bool' }
    ],
    payable: false,
    stateMutability: 'view',
    type: 'function'
  },
  {
    anonymous: false,
    inputs: [
      { indexed: true, internalType: 'bytes32', name: 'channelId', type: 'bytes32' },
      { indexed: false, internalType: 'uint256', name: 'amount', type: 'uint256' },
      { indexed: false, internalType: 'uint256', name: 'time', type: 'uint256' }
    ],
    name: 'openedChannel',
    type: 'event'
  },
  {
    anonymous: false,
    inputs: [
      { indexed: true, internalType: 'bytes32', name: 'channelId', type: 'bytes32' },
      { indexed: false, internalType: 'uint256', name: 'amount', type: 'uint256' },
      { indexed: false, internalType: 'uint256', name: 'time', type: 'uint256' }
    ],
    name: 'closedChannel',
    type: 'event'
  }
];