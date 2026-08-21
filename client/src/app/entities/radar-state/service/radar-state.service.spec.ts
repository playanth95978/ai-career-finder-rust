import { afterEach, beforeEach, describe, expect, it } from 'vitest';
import { HttpTestingController, provideHttpClientTesting } from '@angular/common/http/testing';
import { TestBed } from '@angular/core/testing';

import { IRadarState } from '../radar-state.model';
import { sampleWithFullData, sampleWithNewData, sampleWithPartialData, sampleWithRequiredData } from '../radar-state.test-samples';

import { RadarStateService, RestRadarState } from './radar-state.service';

const requireRestSample: RestRadarState = {
  ...sampleWithRequiredData,
  lastOfferAt: sampleWithRequiredData.lastOfferAt?.toJSON(),
};

describe('RadarState Service', () => {
  let service: RadarStateService;
  let httpMock: HttpTestingController;
  let expectedResult: IRadarState | IRadarState[] | boolean | null;

  beforeEach(() => {
    TestBed.configureTestingModule({
      providers: [provideHttpClientTesting()],
    });
    expectedResult = null;
    service = TestBed.inject(RadarStateService);
    httpMock = TestBed.inject(HttpTestingController);
  });

  describe('Service methods', () => {
    it('should find an element', () => {
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.find(123).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'GET' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should create a RadarState', () => {
      const radarState = { ...sampleWithNewData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.create(radarState).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'POST' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should update a RadarState', () => {
      const radarState = { ...sampleWithRequiredData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.update(radarState).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'PUT' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should partial update a RadarState', () => {
      const patchObject = { ...sampleWithPartialData };
      const returnedFromService = { ...requireRestSample };
      const expected = { ...sampleWithRequiredData };

      service.partialUpdate(patchObject).subscribe(resp => (expectedResult = resp));

      const req = httpMock.expectOne({ method: 'PATCH' });
      req.flush(returnedFromService);
      expect(expectedResult).toMatchObject(expected);
    });

    it('should return a list of RadarState', () => {
      const returnedFromService = { ...requireRestSample };

      const expected = { ...sampleWithRequiredData };

      service.query().subscribe(resp => (expectedResult = resp.body));

      const req = httpMock.expectOne({ method: 'GET' });
      req.flush([returnedFromService]);
      httpMock.verify();
      expect(expectedResult).toMatchObject([expected]);
    });

    it('should delete a RadarState', () => {
      service.delete(123).subscribe();

      const requests = httpMock.match({ method: 'DELETE' });
      expect(requests).toHaveLength(1);
    });

    describe('addRadarStateToCollectionIfMissing', () => {
      it('should add a RadarState to an empty array', () => {
        const radarState: IRadarState = sampleWithRequiredData;
        expectedResult = service.addRadarStateToCollectionIfMissing([], radarState);
        expect(expectedResult).toEqual([radarState]);
      });

      it('should not add a RadarState to an array that contains it', () => {
        const radarState: IRadarState = sampleWithRequiredData;
        const radarStateCollection: IRadarState[] = [
          {
            ...radarState,
          },
          sampleWithPartialData,
        ];
        expectedResult = service.addRadarStateToCollectionIfMissing(radarStateCollection, radarState);
        expect(expectedResult).toHaveLength(2);
      });

      it("should add a RadarState to an array that doesn't contain it", () => {
        const radarState: IRadarState = sampleWithRequiredData;
        const radarStateCollection: IRadarState[] = [sampleWithPartialData];
        expectedResult = service.addRadarStateToCollectionIfMissing(radarStateCollection, radarState);
        expect(expectedResult).toHaveLength(2);
        expect(expectedResult).toContain(radarState);
      });

      it('should add only unique RadarState to an array', () => {
        const radarStateArray: IRadarState[] = [sampleWithRequiredData, sampleWithPartialData, sampleWithFullData];
        const radarStateCollection: IRadarState[] = [sampleWithRequiredData];
        expectedResult = service.addRadarStateToCollectionIfMissing(radarStateCollection, ...radarStateArray);
        expect(expectedResult).toHaveLength(3);
      });

      it('should accept varargs', () => {
        const radarState: IRadarState = sampleWithRequiredData;
        const radarState2: IRadarState = sampleWithPartialData;
        expectedResult = service.addRadarStateToCollectionIfMissing([], radarState, radarState2);
        expect(expectedResult).toEqual([radarState, radarState2]);
      });

      it('should accept null and undefined values', () => {
        const radarState: IRadarState = sampleWithRequiredData;
        expectedResult = service.addRadarStateToCollectionIfMissing([], null, radarState, undefined);
        expect(expectedResult).toEqual([radarState]);
      });

      it('should return initial array if no RadarState is added', () => {
        const radarStateCollection: IRadarState[] = [sampleWithRequiredData];
        expectedResult = service.addRadarStateToCollectionIfMissing(radarStateCollection, undefined, null);
        expect(expectedResult).toEqual(radarStateCollection);
      });
    });

    describe('compareRadarState', () => {
      it('should return true if both entities are null', () => {
        const entity1 = null;
        const entity2 = null;

        const compareResult = service.compareRadarState(entity1, entity2);

        expect(compareResult).toEqual(true);
      });

      it('should return false if one entity is null', () => {
        const entity1 = { id: 23871 };
        const entity2 = null;

        const compareResult1 = service.compareRadarState(entity1, entity2);
        const compareResult2 = service.compareRadarState(entity2, entity1);

        expect(compareResult1).toEqual(false);
        expect(compareResult2).toEqual(false);
      });

      it('should return false if primaryKey differs', () => {
        const entity1 = { id: 23871 };
        const entity2 = { id: 6100 };

        const compareResult1 = service.compareRadarState(entity1, entity2);
        const compareResult2 = service.compareRadarState(entity2, entity1);

        expect(compareResult1).toEqual(false);
        expect(compareResult2).toEqual(false);
      });

      it('should return false if primaryKey matches', () => {
        const entity1 = { id: 23871 };
        const entity2 = { id: 23871 };

        const compareResult1 = service.compareRadarState(entity1, entity2);
        const compareResult2 = service.compareRadarState(entity2, entity1);

        expect(compareResult1).toEqual(true);
        expect(compareResult2).toEqual(true);
      });
    });
  });

  afterEach(() => {
    httpMock.verify();
  });
});
