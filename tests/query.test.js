const axios = require('axios').default;

const QUERY_SERVER = process.env.QUERY_SERVER || "http://localhost:3000";

/** 
 * Make a request to the query server with the given query string. The return 
 * Will be a tuple of ( status_code, body )
 */
const makeRequest = async (query) => {
    const uri = `${QUERY_SERVER}/query${query ? '?' + query : ''}`;

    try {
        const resp = await axios.get(uri);
        return [resp.status, resp.data];
    } catch (error) {
        return [error.response.status, error.response.data];
    }
};

test('Empty Query', async () => {
    const [status, data] = await makeRequest();
    expect(status).toBe(200);
    expect(data).toHaveLength(1461);
});

test('Limited Query', async () => {
    const [status, data] = await makeRequest("limit=5");
    expect(status).toBe(200);
    expect(data).toHaveLength(5);
});

test('Over-Limited Query', async () => {
    const [status, data] = await makeRequest("limit=0");
    expect(status).toBe(200);
    expect(data).toHaveLength(0);
});

test('Specific Date Query', async () => {
    const [status, data] = await makeRequest("date=2012-11-30");
    expect(status).toBe(200);
    expect(data).toHaveLength(1);
    expect(data[0]).toStrictEqual({
        date: "2012-11-30",
        precipitation: 35.6,
        temp_min: 7.8,
        temp_max: 15.0,
        wind: 4.6,
        weather: "rain"
    })
});

test('Snow Query', async () => {
    const [status, data] = await makeRequest("weather=snow");
    expect(status).toBe(200);
    expect(data).not.toHaveLength(0);

    data.forEach(e => expect(e.weather).toBe("snow"));
});

test('Limited Snow Query', async () => {
    const [status, data] = await makeRequest("weather=snow&limit=5");
    expect(status).toBe(200);
    expect(data).toHaveLength(5);

    data.forEach(e => expect(e.weather).toBe("snow"));
});

test('Specific Date and Weather Query', async () => {
    const [status, data] = await makeRequest("date=2012-11-30&weather=rain");
    expect(status).toBe(200);
    expect(data).toHaveLength(1);
    expect(data[0]).toStrictEqual({
        date: "2012-11-30",
        precipitation: 35.6,
        temp_min: 7.8,
        temp_max: 15.0,
        wind: 4.6,
        weather: "rain"
    })
});

test('Specific Date and Weather Query Miss', async () => {
    const [status, data] = await makeRequest("date=2012-11-30&weather=snow");
    expect(status).toBe(200);
    expect(data).toHaveLength(0);
});

test('Specific Date and Weather Query Over Limited', async () => {
    const [status, data] = await makeRequest("date=2012-11-30&weather=rain&limit=0");
    expect(status).toBe(200);
    expect(data).toHaveLength(0);
});

test('Specific Date with No Data', async () => {
    const [status, data] = await makeRequest("date=2000-11-30");
    expect(status).toBe(200);
    expect(data).toHaveLength(0);
});

test('Invalid Date 1', async () => {
    const [status, data] = await makeRequest("date=20123422-11-30");
    expect(status).toBe(400);
    expect(data).toContain('YYYY-MM-DD');
});

test('Invalid Date 2', async () => {
    const [status, data] = await makeRequest("date=yesterday");
    expect(status).toBe(400);
    expect(data).toContain('YYYY-MM-DD');
});

test('Invalid Limit 1', async () => {
    const [status, data] = await makeRequest("limit=");
    expect(status).toBe(400);
    expect(data).toContain('non-negative');
});

test('Invalid Limit 2', async () => {
    const [status, data] = await makeRequest("limit=-10");
    expect(status).toBe(400);
    expect(data).toContain('non-negative');
});

test('Invalid Limit 3', async () => {
    const [status, data] = await makeRequest("limit=seven");
    expect(status).toBe(400);
    expect(data).toContain('non-negative');
});


test('Invalid Weather', async () => {
    const [status, data] = await makeRequest("weather=lava");
    expect(status).toBe(400);
    expect(data).toContain('sun');
    expect(data).toContain('rain');
    expect(data).toContain('snow');
    expect(data).toContain('fog');
    expect(data).toContain('drizzle');
});
