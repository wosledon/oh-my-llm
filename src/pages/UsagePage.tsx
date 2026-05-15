import { useState, useEffect, useCallback } from 'react';
import {
  Card,
  CardHeader,
  CardPreview,
  Table,
  TableHeader,
  TableRow,
  TableHeaderCell,
  TableBody,
  TableCell,
  Text,
  makeStyles,
  tokens,
  Spinner,
  Button,
} from '@fluentui/react-components';
import {
  Calendar24Regular,
} from '@fluentui/react-icons';
import { useI18n } from '../i18n';

interface DailyUsage {
  date: string;
  model: string;
  provider_id: string;
  request_count: number;
  prompt_tokens: number;
  completion_tokens: number;
  cost: number;
}

interface UsageSummary {
  total_requests: number;
  total_prompt_tokens: number;
  total_completion_tokens: number;
  total_cost: number;
}

const useStyles = makeStyles({
  container: {
    display: 'flex',
    flexDirection: 'column',
    gap: tokens.spacingVerticalL,
    padding: tokens.spacingHorizontalL,
  },
  statsGrid: {
    display: 'grid',
    gridTemplateColumns: 'repeat(auto-fill, minmax(200px, 1fr))',
    gap: tokens.spacingHorizontalM,
  },
  statCard: {
    padding: tokens.spacingHorizontalM,
    display: 'flex',
    flexDirection: 'column',
    gap: tokens.spacingVerticalS,
  },
  statValue: {
    fontSize: '24px',
    fontWeight: '600',
    color: tokens.colorBrandForeground1,
  },
  statLabel: {
    fontSize: '12px',
    color: tokens.colorNeutralForeground2,
  },
  toolbar: {
    display: 'flex',
    gap: tokens.spacingHorizontalM,
    alignItems: 'center',
  },
  dateInput: {
    padding: '6px 10px',
    borderRadius: tokens.borderRadiusMedium,
    border: `1px solid ${tokens.colorNeutralStroke1}`,
    backgroundColor: tokens.colorNeutralBackground1,
    color: tokens.colorNeutralForeground1,
  },
});

export default function UsagePage() {
  const { t } = useI18n();
  const styles = useStyles();

  const today = new Date().toISOString().split('T')[0];
  const thirtyDaysAgo = new Date(Date.now() - 30 * 86400000).toISOString().split('T')[0];

  const [startDate, setStartDate] = useState(thirtyDaysAgo);
  const [endDate, setEndDate] = useState(today);
  const [daily, setDaily] = useState<DailyUsage[]>([]);
  const [summary, setSummary] = useState<UsageSummary>({
    total_requests: 0,
    total_prompt_tokens: 0,
    total_completion_tokens: 0,
    total_cost: 0,
  });
  const [loading, setLoading] = useState(false);

  const fetchUsage = useCallback(async () => {
    setLoading(true);
    try {
      const result = await (window as any).__TAURI_INVOKE__('get_usage', {
        params: {
          start_date: startDate,
          end_date: endDate,
        },
      });
      setDaily(result.daily || []);
      setSummary(result.summary || { total_requests: 0, total_prompt_tokens: 0, total_completion_tokens: 0, total_cost: 0 });
    } catch (e) {
      console.error('Failed to fetch usage', e);
    } finally {
      setLoading(false);
    }
  }, [startDate, endDate]);

  useEffect(() => {
    fetchUsage();
  }, [fetchUsage]);

  return (
    <div className={styles.container}>
      <Card>
        <CardHeader
          header={<Text weight="semibold" size={500}>{t.usage.title}</Text>}
        />
        <CardPreview>
          <div style={{ padding: '16px' }}>
            <div className={styles.toolbar} style={{ marginBottom: '16px' }}>
              <Calendar24Regular />
              <input
                type="date"
                className={styles.dateInput}
                value={startDate}
                onChange={(e) => setStartDate(e.target.value)}
              />
              <Text>-&#124;-</Text>
              <input
                type="date"
                className={styles.dateInput}
                value={endDate}
                onChange={(e) => setEndDate(e.target.value)}
              />
              <Button appearance="primary" onClick={fetchUsage}>{t.common.confirm}</Button>
            </div>

            <div className={styles.statsGrid}>
              <Card className={styles.statCard}>
                <Text className={styles.statValue}>{summary.total_requests}</Text>
                <Text className={styles.statLabel}>{t.usage.totalRequests}</Text>
              </Card>
              <Card className={styles.statCard}>
                <Text className={styles.statValue}>{summary.total_prompt_tokens.toLocaleString()}</Text>
                <Text className={styles.statLabel}>{t.usage.totalPromptTokens}</Text>
              </Card>
              <Card className={styles.statCard}>
                <Text className={styles.statValue}>{summary.total_completion_tokens.toLocaleString()}</Text>
                <Text className={styles.statLabel}>{t.usage.totalCompletionTokens}</Text>
              </Card>
              <Card className={styles.statCard}>
                <Text className={styles.statValue}>${summary.total_cost.toFixed(4)}</Text>
                <Text className={styles.statLabel}>{t.usage.totalCost}</Text>
              </Card>
            </div>

            <Text weight="semibold" size={400} style={{ marginTop: '24px', marginBottom: '12px' }}>
              {t.usage.dailyBreakdown}
            </Text>

            {loading ? (
              <Spinner style={{ margin: '40px auto', display: 'block' }} />
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHeaderCell>{t.usage.dateRange}</TableHeaderCell>
                    <TableHeaderCell>{t.usage.model}</TableHeaderCell>
                    <TableHeaderCell>{t.usage.requestCount}</TableHeaderCell>
                    <TableHeaderCell>{t.usage.promptTokens}</TableHeaderCell>
                    <TableHeaderCell>{t.usage.completionTokens}</TableHeaderCell>
                    <TableHeaderCell>{t.usage.cost}</TableHeaderCell>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {daily.map((item, idx) => (
                    <TableRow key={idx}>
                      <TableCell>{item.date}</TableCell>
                      <TableCell>{item.model}</TableCell>
                      <TableCell>{item.request_count}</TableCell>
                      <TableCell>{item.prompt_tokens.toLocaleString()}</TableCell>
                      <TableCell>{item.completion_tokens.toLocaleString()}</TableCell>
                      <TableCell>${item.cost.toFixed(4)}</TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            )}
          </div>
        </CardPreview>
      </Card>
    </div>
  );
}
